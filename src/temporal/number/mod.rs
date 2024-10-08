pub mod tfloat;
pub mod tint;
pub mod tnumber;

#[cfg(test)]
mod tests {
    use crate::meos_initialize;

    use super::*;

    #[test]
    fn instant_tint() {
        meos_initialize("UTC");
        let string = "1@2018-01-01 08:00:00+00";
        let result: tint::TInt = string.parse().unwrap();
        assert_eq!(
            format!("{result:?}"),
            format!("Instant({})", string.to_owned())
        );
    }

    #[test]
    fn sequence_tint() {
        meos_initialize("UTC");
        let string = "[1@2018-01-01 08:00:00+00]";
        let result: tint::TInt = string.parse().unwrap();
        assert_eq!(
            format!("{result:?}"),
            format!("Sequence({})", string.to_owned())
        );
    }

    #[test]
    fn sequence_set_tint() {
        meos_initialize("UTC");
        let string = "{[1@2018-01-01 08:00:00+00]}";
        let result: tint::TInt = string.parse().unwrap();
        assert_eq!(
            format!("{result:?}"),
            format!("SequenceSet({})", string.to_owned())
        );
    }

    #[test]
    fn instant_tfloat() {
        meos_initialize("UTC");
        let string = "1@2018-01-01 08:00:00+00";
        let result: tfloat::TFloat = string.parse().unwrap();
        assert_eq!(
            format!("{result:?}"),
            format!("Instant({})", string.to_owned())
        );
    }

    #[test]
    fn sequence_tfloat() {
        meos_initialize("UTC");
        let string = "[1@2018-01-01 08:00:00+00]";
        let result: tfloat::TFloat = string.parse().unwrap();
        assert_eq!(
            format!("{result:?}"),
            format!("Sequence({})", string.to_owned())
        );
    }

    #[test]
    fn sequence_set_tfloat() {
        meos_initialize("UTC");
        let string = "{[1@2018-01-01 08:00:00+00]}";
        let result: tfloat::TFloat = string.parse().unwrap();
        assert_eq!(
            format!("{result:?}"),
            format!("SequenceSet({})", string.to_owned())
        );
    }
}
