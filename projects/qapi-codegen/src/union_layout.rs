//! The JSON members of object types as QEMU's generator lays them out from
//! 2.6, to find the clashes and cycles among them: a struct's base and own
//! members, a union's base and, branch by branch, the members of its
//! branch type, and the types that QEMU makes of members written out in
//! place. The walk keeps its own stack, so a chain of bases as long as the
//! generator takes needs no deeper call stack.

use crate::schema_validation::{Meta, Schema, since, spelled};
use crate::unified::{NameRule, SchemaContext, SchemaErrorKind, SchemaResult, require};
use crate::{MembersOrRef, QapiMember, QapiMembers, QapiSchemaToken, QapiTypeRef, QapiUnion};
use crate::{QapiVariant, schema_names};
use std::collections::HashMap;
use std::rc::Rc;

/// A member, enum value, branch name or feature, with the definition that
/// declares it. Its parts are shared, as each object that inherits it
/// holds it too.
#[derive(Clone, Debug)]
pub(crate) struct Member {
    name: Rc<str>,
    /// Its C name, which a clash compares.
    key: Rc<str>,
    owner: Rc<str>,
    pub(crate) conditional: bool,
    optional: bool,
    /// The type, and whether the member is an array of it.
    r#type: Option<(Rc<str>, bool)>,
}

impl Member {
    pub(crate) fn new(name: &str, owner: &str) -> Self {
        Self {
            name: name.into(),
            key: schema_names::c_name(name).into(),
            owner: owner.into(),
            conditional: false,
            optional: false,
            r#type: None,
        }
    }

    fn field(field: &QapiMember<'_>, owner: &str) -> Self {
        Self {
            conditional: field.r#if.is_some(),
            optional: field.optional,
            r#type: match &field.r#type {
                QapiTypeRef::Ref(name) => Some((name.as_ref().into(), false)),
                QapiTypeRef::ArrayRef(name) => Some((name.as_ref().into(), true)),
                _ => None,
            },
            ..Self::new(&field.name, owner)
        }
    }
}

/// The members seen so far, by C name.
pub(crate) type Seen = HashMap<Rc<str>, Member>;

/// A checked struct or union.
pub(crate) struct Object {
    /// Its members outside its branches, inherited ones first.
    pub(crate) members: Vec<Member>,
    /// The same, by C name.
    seen: Seen,
    /// Whether it is a union.
    pub(crate) branches: bool,
    /// The object types of its branches.
    variants: Vec<Rc<Object>>,
}

enum State {
    Checking,
    Done(Rc<Object>),
}

/// The structs, unions and implicit object types checked so far.
#[derive(Default)]
pub(crate) struct Objects {
    states: HashMap<String, State>,
}

/// Record `member` in `seen`, refusing a clash of C names; from QEMU 2.6 to
/// 4.1 also uppercase, unless the release exempts the member's owner.
pub(crate) fn clash(schema: &Schema<'_, '_>, seen: &mut Seen, member: Member) -> SchemaResult {
    if since((2, 6, 0)) && !since((4, 2, 0)) {
        require(
            !member.name.contains(|c: char| c.is_ascii_uppercase())
                || schema.case_exempt(&member.owner),
            || SchemaErrorKind::NameCase {
                name: member.name.to_string(),
                rule: NameRule::NoUppercase,
            },
        )?;
    }
    if let Some(previous) = seen.get(&member.key) {
        return Err(SchemaErrorKind::MemberClash {
            name: member.name.to_string(),
            previous: previous.name.to_string(),
        }
        .into());
    }
    seen.insert(member.key.clone(), member);
    Ok(())
}

/// A step of QEMU's check of an object type, in its order.
enum Step<'a, 's> {
    /// The definition's own features.
    Features(&'a QapiSchemaToken<'s>),
    /// From 4.2, that the base is a struct.
    ResolveBase(&'a str),
    /// The check of another object type: a base or a branch type.
    Check(String),
    /// The members of the base just checked.
    TakeBase,
    /// Members that `owner` declares itself.
    Local(&'a QapiMembers<'s>, String),
    /// A simple union's tag member, and from 4.2 that it has branches.
    Tag(&'a QapiUnion<'s>),
    /// From 4.2, a flat union's discriminator and that it has branches.
    Discriminator(&'a str, &'a QapiUnion<'s>),
    /// A flat union's branch.
    Branch(&'a QapiVariant<'s>),
    /// The branch type just checked.
    Variant,
    /// The members of each branch type against those of the union.
    Variants,
    /// Before 4.2, the documentation of the definition.
    Doc,
}

/// An object type being checked.
struct Frame<'a, 's> {
    name: String,
    /// The definition that its errors belong to.
    index: usize,
    /// The steps still to take, the next one last.
    steps: Vec<Step<'a, 's>>,
    seen: Seen,
    members: Vec<Member>,
    branches: bool,
    variants: Vec<Rc<Object>>,
    /// A flat union's discriminator enum and its values, from 4.2.
    tags: Option<(String, Vec<&'a str>)>,
    /// The object type that the last `Check` step checked.
    checked: Option<Rc<Object>>,
}

impl<'a, 's> Frame<'a, 's> {
    /// The steps of the object type `name`, which belongs to the
    /// definition at `index`.
    fn new(schema: &Schema<'a, 's>, name: &str, index: usize) -> Self {
        let modern = since((4, 2, 0));
        let mut steps = Vec::new();
        let mut branches = false;
        if let Some((_, fields)) = schema.implicit(name) {
            steps.push(Step::Local(fields, name.to_owned()));
        } else {
            match schema.token(name) {
                Some(token @ QapiSchemaToken::Struct(value)) => {
                    if modern {
                        steps.push(Step::Features(token));
                    }
                    if let Some(base) = value.base.as_deref() {
                        if modern {
                            steps.push(Step::ResolveBase(base));
                        }
                        steps.extend([Step::Check(base.to_owned()), Step::TakeBase]);
                    }
                    steps.push(Step::Local(&value.data, name.to_owned()));
                }
                Some(token @ QapiSchemaToken::Union(value)) => {
                    if modern {
                        steps.push(Step::Features(token));
                    }
                    match &value.base {
                        MembersOrRef::Ref(base) => {
                            if modern {
                                steps.push(Step::ResolveBase(base));
                            }
                            steps.extend([Step::Check(base.to_string()), Step::TakeBase]);
                        }
                        MembersOrRef::Members(fields) if !fields.0.is_empty() => {
                            steps.extend([
                                Step::Check(format!("q_obj_{name}-base")),
                                Step::TakeBase,
                            ]);
                        }
                        _ => {}
                    }
                    match value.discriminator.as_deref() {
                        _ if value.legacy_alternate => {}
                        None => {
                            branches = true;
                            steps.push(Step::Tag(value));
                        }
                        Some(discriminator) => {
                            branches = true;
                            if modern {
                                steps.push(Step::Discriminator(discriminator, value));
                            }
                            steps.extend(value.data.0.iter().map(Step::Branch));
                            steps.push(Step::Variants);
                        }
                    }
                }
                _ => {}
            }
            // 4.1 checks a struct's features after its members, and before
            // 4.2 the documentation with the type.
            if let Some(token @ (QapiSchemaToken::Struct(_) | QapiSchemaToken::Union(_))) =
                schema.token(name)
                && !modern
            {
                if since((4, 1, 0)) {
                    steps.push(Step::Features(token));
                }
                steps.push(Step::Doc);
            }
        }
        steps.reverse();
        Self {
            name: name.to_owned(),
            index,
            steps,
            seen: HashMap::new(),
            members: Vec::new(),
            branches,
            variants: Vec::new(),
            tags: None,
            checked: None,
        }
    }

    /// Take one step; `Some` names an object type to check next.
    fn step(
        &mut self,
        schema: &Schema<'a, 's>,
        step: Step<'a, 's>,
    ) -> SchemaResult<Option<String>> {
        match step {
            Step::Features(token) => schema.definition_features(token)?,
            Step::ResolveBase(base) => {
                let meta = schema
                    .resolve(&QapiTypeRef::Ref(base.into()))
                    .within(|| "'base'".into())?;
                require(meta == Meta::Struct, || SchemaErrorKind::TypeNotAllowed {
                    meta: meta.word(),
                    reference: format!("'{base}'"),
                })
                .within(|| "'base'".into())?;
            }
            Step::Check(name) => return Ok(Some(name)),
            // The base's members, which its own check found free of
            // clashes, come first.
            Step::TakeBase => {
                if let Some(base) = self.checked.take() {
                    debug_assert!(self.members.is_empty());
                    self.seen.clone_from(&base.seen);
                    self.members.clone_from(&base.members);
                }
            }
            Step::Local(fields, owner) => {
                for field in &fields.0 {
                    let check = || -> SchemaResult {
                        if since((4, 2, 0)) {
                            schema.resolve(&field.r#type)?;
                        } else if let QapiTypeRef::Ref(name) = &field.r#type {
                            schema.alternate_tag(name)?;
                        }
                        if since((5, 0, 0)) {
                            schema.features_clash(field.features.as_ref(), &field.name)?;
                        }
                        Ok(())
                    };
                    check().within(|| format!("member '{}'", field.name))?;
                    let member = Member::field(field, &owner);
                    clash(schema, &mut self.seen, member.clone())?;
                    self.members.push(member);
                }
            }
            Step::Tag(value) => {
                // A simple union's branches wrap their types as member
                // `data`; the tag enum checks the branch names.
                let tag = Member::new("type", &self.name);
                clash(schema, &mut self.seen, tag.clone())?;
                self.members.push(tag);
                require(!value.data.0.is_empty() || !since((4, 2, 0)), || {
                    SchemaErrorKind::NoBranches
                })?;
            }
            Step::Discriminator(discriminator, value) => {
                let (enumeration, values) =
                    discriminator_enum(schema, &self.members, discriminator)?;
                require(!value.data.0.is_empty() || !values.is_empty(), || {
                    SchemaErrorKind::NoBranches
                })?;
                self.tags = Some((enumeration, values));
            }
            Step::Branch(branch) => {
                if let Some((enumeration, values)) = &self.tags {
                    let meta = schema
                        .resolve(&branch.r#type)
                        .within(|| format!("branch '{}'", branch.name))?;
                    schema.branch_value(&branch.name, enumeration, values)?;
                    let object = meta == Meta::Struct || meta == Meta::Union && since((8, 1, 0));
                    require(
                        object && matches!(branch.r#type, QapiTypeRef::Ref(_)),
                        || SchemaErrorKind::TypeNotAllowed {
                            meta: meta.word(),
                            reference: spelled(&branch.r#type),
                        },
                    )
                    .within(|| format!("branch '{}'", branch.name))?;
                }
                if let QapiTypeRef::Ref(branch_type) = &branch.r#type
                    && matches!(schema.meta(branch_type), Some(Meta::Struct | Meta::Union))
                {
                    self.steps
                        .extend([Step::Variant, Step::Check(branch_type.to_string())]);
                }
            }
            Step::Variant => self.variants.extend(self.checked.take()),
            Step::Variants => {
                // Each branch type, and in turn each of its branches, with
                // the members so far: depth first, in order.
                let mut pending: Vec<_> = self
                    .variants
                    .iter()
                    .rev()
                    .map(|variant| (variant.clone(), self.seen.clone()))
                    .collect();
                while let Some((object, mut seen)) = pending.pop() {
                    for member in &object.members {
                        clash(schema, &mut seen, member.clone())?;
                    }
                    for nested in object.variants.iter().rev() {
                        pending.push((nested.clone(), seen.clone()));
                    }
                }
            }
            Step::Doc => schema.described(self.index)?,
        }
        Ok(None)
    }
}

/// Find a flat union's discriminator among the members of its base, as
/// QEMU does from 4.2, and the values of its enum.
fn discriminator_enum<'a>(
    schema: &Schema<'a, '_>,
    members: &[Member],
    discriminator: &str,
) -> SchemaResult<(String, Vec<&'a str>)> {
    let key = schema_names::c_name(discriminator);
    let tag = members
        .iter()
        .find(|member| *member.key == *key)
        .filter(|member| &*member.name == discriminator)
        .ok_or_else(|| SchemaErrorKind::DiscriminatorNotMember {
            discriminator: discriminator.to_owned(),
        })?;
    let (enumeration, array) = tag
        .r#type
        .as_ref()
        .map(|(name, array)| (name.to_string(), *array))
        .unwrap_or_default();
    let values = (!array)
        .then(|| schema.enum_values(&enumeration))
        .flatten()
        .ok_or_else(|| SchemaErrorKind::DiscriminatorNotEnum {
            discriminator: discriminator.to_owned(),
            discriminator_type: if array {
                format!("['{enumeration}']")
            } else {
                enumeration.clone()
            },
        })?;
    require(!tag.optional, || SchemaErrorKind::OptionalDiscriminator {
        discriminator: discriminator.to_owned(),
    })?;
    require(!tag.conditional, || {
        SchemaErrorKind::ConditionalDiscriminator {
            discriminator: discriminator.to_owned(),
        }
    })?;
    Ok((enumeration, values))
}

impl Objects {
    /// Check the object type `name` once, with the types it uses: a
    /// struct, a union or an implicit type of members written out in
    /// place. An error is located at the definition it belongs to.
    pub(crate) fn check(
        &mut self,
        schema: &Schema<'_, '_>,
        name: &str,
    ) -> SchemaResult<Rc<Object>> {
        if let Some(State::Done(object)) = self.states.get(name) {
            return Ok(object.clone());
        }
        let mut stack = vec![self.start(schema, name)?];
        while let Some(frame) = stack.last_mut() {
            let Some(step) = frame.steps.pop() else {
                let Some(frame) = stack.pop() else { break };
                let object = Rc::new(Object {
                    members: frame.members,
                    seen: frame.seen,
                    branches: frame.branches,
                    variants: frame.variants,
                });
                self.states.insert(frame.name, State::Done(object.clone()));
                match stack.last_mut() {
                    Some(parent) => parent.checked = Some(object),
                    None => return Ok(object),
                }
                continue;
            };
            let index = frame.index;
            let next = schema.located(index, frame.step(schema, step))?;
            if let Some(next) = next {
                match self.states.get(&next) {
                    Some(State::Done(object)) => frame.checked = Some(object.clone()),
                    _ => {
                        let next = schema.located(index, self.start(schema, &next))?;
                        stack.push(next);
                    }
                }
            }
        }
        Err(SchemaErrorKind::UnknownType {
            name: name.to_owned(),
        }
        .into())
    }

    /// Begin the check of `name`; an object type being checked already
    /// contains itself.
    fn start<'a, 's>(
        &mut self,
        schema: &Schema<'a, 's>,
        name: &str,
    ) -> SchemaResult<Frame<'a, 's>> {
        let index = match schema.implicit(name) {
            Some((owner, _)) => owner,
            None => schema
                .index(name)
                .ok_or_else(|| SchemaErrorKind::UnknownType {
                    name: name.to_owned(),
                })?,
        };
        if let Some(State::Checking) = self.states.get(name) {
            return schema.located(
                index,
                Err(SchemaErrorKind::InheritanceCycle {
                    name: name.to_owned(),
                }
                .into()),
            );
        }
        self.states.insert(name.to_owned(), State::Checking);
        Ok(Frame::new(schema, name, index))
    }
}
