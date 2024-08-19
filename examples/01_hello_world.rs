use meos::{
    init,
    temporal::point::tpoint::{
        TGeomPoint, TGeomPointInstant, TGeomPointSequence, TGeomPointSequenceSet,
    },
    MeosEnum,
};

fn main() {
    init();
    // Input temporal points in WKT format
    let inst_wkt = "POINT(1 1)@2000-01-01";
    let seq_disc_wkt = "{POINT(1 1)@2000-01-01, POINT(2 2)@2000-01-02}";
    let seq_linear_wkt = "[POINT(1 1)@2000-01-01, POINT(2 2)@2000-01-02]";
    let seq_step_wkt = "Interp=Step;[POINT(1 1)@2000-01-01, POINT(2 2)@2000-01-02]";
    let ss_linear_wkt = "{[POINT(1 1)@2000-01-01, POINT(2 2)@2000-01-02],\
                            [POINT(3 3)@2000-01-03, POINT(3 3)@2000-01-04]}";
    let ss_step_wkt = "Interp=Step;{[POINT(1 1)@2000-01-01, POINT(2 2)@2000-01-02],\
                          [POINT(3 3)@2000-01-03, POINT(3 3)@2000-01-04]}";

    // Read WKT into temporal point objects
    let inst: TGeomPointInstant = inst_wkt.parse().unwrap();
    let seq_disc: TGeomPointSequence = seq_disc_wkt.parse().unwrap();
    let seq_linear: TGeomPointSequence = seq_linear_wkt.parse().unwrap();
    let seq_step: TGeomPointSequence = seq_step_wkt.parse().unwrap();
    let ss_linear: TGeomPointSequenceSet = ss_linear_wkt.parse().unwrap();
    let ss_step: TGeomPointSequenceSet = ss_step_wkt.parse().unwrap();

    // Convert results to MF-JSON

    let inst_mfjson =
        TGeomPoint::Instant(inst).as_mfjson(true, meos::temporal::JSONCVariant::Pretty, 6, "4326");
    println!(
        "\n\
            --------------------\n\
            | Temporal Instant |\n\
            --------------------\n\n\
            WKT:\n\
            ----\n{}\n\n\
            MF-JSON:\n\
            --------\n{}",
        inst_wkt, inst_mfjson
    );

    let seq_disc_mfjson = TGeomPoint::Sequence(seq_disc).as_mfjson(
        true,
        meos::temporal::JSONCVariant::Pretty,
        6,
        "4326",
    );
    println!(
        "\n\
            -------------------------------------------------\n\
            | Temporal Sequence with Discrete Interpolation |\n\
            -------------------------------------------------\n\
            WKT:\n\
            ----\n{}\n\n\
            MF-JSON:\n\
            --------\n{}",
        seq_disc_wkt, seq_disc_mfjson
    );

    let seq_linear_mfjson = TGeomPoint::Sequence(seq_linear).as_mfjson(
        true,
        meos::temporal::JSONCVariant::Pretty,
        6,
        "4326",
    );
    println!(
        "\n\
            -----------------------------------------------\n\
            | Temporal Sequence with Linear Interpolation |\n\
            -----------------------------------------------\n\
            WKT:\n\
            ----\n{}\n\n\
            MF-JSON:\n\
            --------\n{}",
        seq_linear_wkt, seq_linear_mfjson
    );

    let seq_step_mfjson = TGeomPoint::Sequence(seq_step).as_mfjson(
        true,
        meos::temporal::JSONCVariant::Pretty,
        6,
        "4326",
    );
    println!(
        "\n\
            --------------------------------------------\n\
            | Temporal Sequence with Step Interpolation |\n\
            --------------------------------------------\n\
            WKT:\n\
            ----\n{}\n\n\
            MF-JSON:\n\
            --------\n{}",
        seq_step_wkt, seq_step_mfjson
    );

    let ss_linear_mfjson = TGeomPoint::SequenceSet(ss_linear).as_mfjson(
        true,
        meos::temporal::JSONCVariant::Pretty,
        6,
        "4326",
    );
    println!(
        "\n\
            ---------------------------------------------------\n\
            | Temporal Sequence Set with Linear Interpolation |\n\
            ---------------------------------------------------\n\
            WKT:\n\
            ----\n{}\n\n\
            MF-JSON:\n\
            --------\n{}",
        ss_linear_wkt, ss_linear_mfjson
    );

    let ss_step_mfjson = TGeomPoint::SequenceSet(ss_step).as_mfjson(
        true,
        meos::temporal::JSONCVariant::Pretty,
        6,
        "4326",
    );
    println!(
        "\n\
            ------------------------------------------------\n\
            | Temporal Sequence Set with Step Interpolation |\n\
            ------------------------------------------------\n\
            WKT:\n\
            ----\n{}\n\n\
            MF-JSON:\n\
            --------\n{}",
        ss_step_wkt, ss_step_mfjson
    );
}
