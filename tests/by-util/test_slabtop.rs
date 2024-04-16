use crate::common::util::TestScenario;
use crate::test_slabtop::parse::parse_data;
use crate::test_slabtop::parse::parse_meta;
use crate::test_slabtop::parse::parse_version;
use crate::test_slabtop::parse::SlabInfo;

#[path = "../../src/uu/slabtop/src/parse.rs"]
mod parse;

#[test]
fn test_invalid_arg() {
    new_ucmd!().arg("--definitely-invalid").fails().code_is(1);
}

#[test]
fn test_slabtop() {
    new_ucmd!().arg("--help").succeeds().code_is(0);
}

#[test]
fn test_parse_version() {
    let test = "slabinfo - version: 2.1";
    assert_eq!("2.1", parse_version(test).unwrap())
}

#[test]
fn test_parse_meta() {
    let test="# name            <active_objs> <num_objs> <objsize> <objperslab> <pagesperslab> : tunables <limit> <batchcount> <sharedfactor> : slabdata <active_slabs> <num_slabs> <sharedavail>";

    let result = parse_meta(test);

    assert_eq!(
        result,
        [
            "active_objs",
            "num_objs",
            "objsize",
            "objperslab",
            "pagesperslab",
            "limit",
            "batchcount",
            "sharedfactor",
            "active_slabs",
            "num_slabs",
            "sharedavail"
        ]
    )
}

#[test]
fn test_parse_data() {
    // Success case

    let test = "nf_conntrack_expect      0      0    208   39    2 : tunables    0    0    0 : slabdata      0      0      0";
    let (name, value) = parse_data(test).unwrap();

    assert_eq!(name, "nf_conntrack_expect");
    assert_eq!(value, [0, 0, 208, 39, 2, 0, 0, 0, 0, 0, 0]);

    // Fail case
    let test =
        "0      0    208   39    2 : tunables    0    0    0 : slabdata      0      0      0";
    let (name, _value) = parse_data(test).unwrap();

    assert_ne!(name, "nf_conntrack_expect");
}

#[test]
fn test_parse() {
    let test = include_str!("../fixtures/slabtop/data.txt");
    let result = SlabInfo::parse(test.into()).unwrap();

    assert_eq!(result.fetch("nf_conntrack_expect", "objsize").unwrap(), 208);
    assert_eq!(
        result.fetch("dmaengine-unmap-2", "active_slabs").unwrap(),
        16389
    );
}
