use super::qapi_variant::QapiVariant;
use nom::IResult;

pub(crate) type QapiAlternative<'i> = QapiVariant<'i>;

pub(crate) fn take_alternative(
    input: &str,
) -> IResult<&str, QapiAlternative<'_>> {
    QapiVariant::parse(input)
}
