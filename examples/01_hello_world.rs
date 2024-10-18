use meos::{meos_initialize, MeosEnum, TGeomPoint};

fn main() {
    meos_initialize();
    // Input temporal points in WKT format
    let instant_wkt = "POINT(1 1)@2000-01-01";
    let sequence_discrete_wkt = "{POINT(1 1)@2000-01-01, POINT(2 2)@2000-01-02}";
    let sequence_linear_wkt = "[POINT(1 1)@2000-01-01, POINT(2 2)@2000-01-02]";
    let sequence_step_wkt = "Interp=Step;[POINT(1 1)@2000-01-01, POINT(2 2)@2000-01-02]";
    let sequence_set_linear_wkt = "{[POINT(1 1)@2000-01-01, POINT(2 2)@2000-01-02],\
                            [POINT(3 3)@2000-01-03, POINT(3 3)@2000-01-04]}";
    let sequence_set_step_wkt = "Interp=Step;{[POINT(1 1)@2000-01-01, POINT(2 2)@2000-01-02],\
                          [POINT(3 3)@2000-01-03, POINT(3 3)@2000-01-04]}";

    // Read WKT into temporal point objects
    let TGeomPoint::Instant(inst) = instant_wkt.parse().unwrap() else {
        panic!()
    };
    let TGeomPoint::Sequence(seq_disc) = sequence_discrete_wkt.parse().unwrap() else {
        panic!()
    };
    let TGeomPoint::Sequence(seq_linear) = sequence_linear_wkt.parse().unwrap() else {
        panic!()
    };
    let TGeomPoint::Sequence(seq_step) = sequence_step_wkt.parse().unwrap() else {
        panic!()
    };
    let TGeomPoint::SequenceSet(ss_linear) = sequence_set_linear_wkt.parse().unwrap() else {
        panic!()
    };
    let TGeomPoint::SequenceSet(ss_step) = sequence_set_step_wkt.parse().unwrap() else {
        panic!()
    };

    // Convert results to MF-JSON

    let instant_mfjson =
        TGeomPoint::Instant(inst).as_mfjson(true, meos::JSONCVariant::Pretty, 6, "4326");
    println!(
        "\n\
            --------------------\n\
            | Temporal Instant |\n\
            --------------------\n\n\
            WKT:\n\
            ----\n{}\n\n\
            MF-JSON:\n\
            --------\n{}",
        instant_wkt, instant_mfjson
    );

    let seq_disc_mfjson =
        TGeomPoint::Sequence(seq_disc).as_mfjson(true, meos::JSONCVariant::Pretty, 6, "4326");
    println!(
        "\n\
            -------------------------------------------------\n\
            | Temporal Sequence with Discrete Interpolation |\n\
            -------------------------------------------------\n\
            WKT:\n\
            ----\n{}\n\n\
            MF-JSON:\n\
            --------\n{}",
        sequence_discrete_wkt, seq_disc_mfjson
    );

    let seq_linear_mfjson =
        TGeomPoint::Sequence(seq_linear).as_mfjson(true, meos::JSONCVariant::Pretty, 6, "4326");
    println!(
        "\n\
            -----------------------------------------------\n\
            | Temporal Sequence with Linear Interpolation |\n\
            -----------------------------------------------\n\
            WKT:\n\
            ----\n{}\n\n\
            MF-JSON:\n\
            --------\n{}",
        sequence_linear_wkt, seq_linear_mfjson
    );

    let seq_step_mfjson =
        TGeomPoint::Sequence(seq_step).as_mfjson(true, meos::JSONCVariant::Pretty, 6, "4326");
    println!(
        "\n\
            --------------------------------------------\n\
            | Temporal Sequence with Step Interpolation |\n\
            --------------------------------------------\n\
            WKT:\n\
            ----\n{}\n\n\
            MF-JSON:\n\
            --------\n{}",
        sequence_step_wkt, seq_step_mfjson
    );

    let ss_linear_mfjson =
        TGeomPoint::SequenceSet(ss_linear).as_mfjson(true, meos::JSONCVariant::Pretty, 6, "4326");
    println!(
        "\n\
            ---------------------------------------------------\n\
            | Temporal Sequence Set with Linear Interpolation |\n\
            ---------------------------------------------------\n\
            WKT:\n\
            ----\n{}\n\n\
            MF-JSON:\n\
            --------\n{}",
        sequence_set_linear_wkt, ss_linear_mfjson
    );

    let ss_step_mfjson =
        TGeomPoint::SequenceSet(ss_step).as_mfjson(true, meos::JSONCVariant::Pretty, 6, "4326");
    println!(
        "\n\
            ------------------------------------------------\n\
            | Temporal Sequence Set with Step Interpolation |\n\
            ------------------------------------------------\n\
            WKT:\n\
            ----\n{}\n\n\
            MF-JSON:\n\
            --------\n{}",
        sequence_set_step_wkt, ss_step_mfjson
    );
}
