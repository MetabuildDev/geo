use crate::{MultiPolygon, Polygon};

use log::{error, info};

use std::{
    error::Error,
    panic::{catch_unwind, resume_unwind},
};
use wkt::{ToWkt, TryFromWkt};

pub(super) fn init_log() {
    use pretty_env_logger::env_logger;
    use std::io::Write;
    let _ = env_logger::builder()
        .format(|buf, record| writeln!(buf, "{} - {}", record.level(), record.args()))
        .try_init();
}

use super::*;
type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[test]
fn test_rect_overlapping() -> Result<()> {
    // Two rects that overlap
    let wkt1 = "POLYGON((0 0,1 0,1 1,0 1,0 0))";
    let wkt2 = "POLYGON((0.5 1,2 1,2 2,0.5 2,0.5 1))";

    let wkt_union = "MULTIPOLYGON(((2 1,1 1,1 0,0 0,0 1,0.5 1,0.5 2,2 2,2 1)))";
    let output = check_sweep(wkt1, wkt2, OpType::Union)?;
    assert_eq!(output, MultiPolygon::try_from_wkt_str(wkt_union).unwrap());
    Ok(())
}

#[test]
fn test_ext_in_hole() -> Result<()> {
    // A union which outputs a ring inside a hole inside a ext.
    let wkt1 = "POLYGON((0 0, 40 0, 40 40, 0 40, 0 0), (10 10, 30 10, 30 30, 10 30, 10 10))";
    let wkt2 = "POLYGON((11 11, 29 11, 29 29, 11 29, 11 11), (15 15, 25 15, 25 25, 15 25, 15 15))";
    check_sweep(wkt1, wkt2, OpType::Union)?;
    Ok(())
}

#[test]
fn test_invalid_simple() -> Result<()> {
    // Polygon with holes and invalid
    let wkt1 = "POLYGON((0 0, 2 2, 2 0, 0 0), (1 1, 2 1, 1 0))";
    let wkt2 = "POLYGON EMPTY";
    check_sweep(wkt1, wkt2, OpType::Union)?;
    Ok(())
}

#[test]
fn test_invalid_loops() -> Result<()> {
    let wkt1 = "POLYGON((0 0, 2 2, 0 4, -2 2, 0 0, 1 2, 0 3, -1 2, 0 0))";
    let wkt2 = "POLYGON EMPTY";
    check_sweep(wkt1, wkt2, OpType::Union)?;
    Ok(())
}

fn check_sweep(wkt1: &str, wkt2: &str, ty: OpType) -> Result<MultiPolygon<f64>> {
    init_log();
    let poly1 = MultiPolygon::<f64>::try_from_wkt_str(wkt1)
        .or_else(|_| Polygon::<f64>::try_from_wkt_str(wkt1).map(MultiPolygon::from))
        .unwrap();
    let poly2 = MultiPolygon::try_from_wkt_str(wkt2)
        .or_else(|_| Polygon::<f64>::try_from_wkt_str(wkt2).map(MultiPolygon::from))
        .unwrap();
    let mut bop = Op::new(ty, 0);
    bop.add_multi_polygon(&poly1, true);
    bop.add_multi_polygon(&poly2, false);

    let rings = bop.sweep()?;
    info!("Got {n} rings", n = rings.len());
    for ring in rings.iter() {
        info!(
            "\t{hole}: {wkt}",
            wkt = ring.coords().to_wkt(),
            hole = if ring.is_hole() { "HOLE" } else { "EXTR" }
        );
    }

    let polygons = assemble(rings)?;
    info!("got {n} output polygons", n = polygons.len());
    for p in polygons.iter() {
        info!("\t{wkt}", wkt = p.to_wkt());
    }
    Ok(MultiPolygon::new(polygons))
}

#[test]
fn test_complex_rects() -> Result<()> {
    let wkt1 = "MULTIPOLYGON(((-1 -2,-1.0000000000000002 2,-0.8823529411764707 2,-0.8823529411764706 -2,-1 -2)),((-0.7647058823529411 -2,-0.7647058823529412 2,-0.6470588235294118 2,-0.6470588235294118 -2,-0.7647058823529411 -2)),((-0.5294117647058824 -2,-0.5294117647058825 2,-0.41176470588235287 2,-0.4117647058823529 -2,-0.5294117647058824 -2)),((-0.2941176470588236 -2,-0.2941176470588236 2,-0.17647058823529418 2,-0.17647058823529416 -2,-0.2941176470588236 -2)),((-0.05882352941176472 -2,-0.05882352941176472 2,0.05882352941176472 2,0.05882352941176472 -2,-0.05882352941176472 -2)),((0.17647058823529416 -2,0.17647058823529416 2,0.29411764705882365 2,0.2941176470588236 -2,0.17647058823529416 -2)),((0.4117647058823528 -2,0.41176470588235287 2,0.5294117647058821 2,0.5294117647058822 -2,0.4117647058823528 -2)),((0.6470588235294117 -2,0.6470588235294118 2,0.7647058823529411 2,0.7647058823529411 -2,0.6470588235294117 -2)),((0.8823529411764706 -2,0.8823529411764707 2,1.0000000000000002 2,1 -2,0.8823529411764706 -2)))";
    let wkt2 = "MULTIPOLYGON(((-2 -1,2 -1.0000000000000002,2 -0.8823529411764707,-2 -0.8823529411764706,-2 -1)),((-2 -0.7647058823529411,2 -0.7647058823529412,2 -0.6470588235294118,-2 -0.6470588235294118,-2 -0.7647058823529411)),((-2 -0.5294117647058824,2 -0.5294117647058825,2 -0.41176470588235287,-2 -0.4117647058823529,-2 -0.5294117647058824)),((-2 -0.2941176470588236,2 -0.2941176470588236,2 -0.17647058823529418,-2 -0.17647058823529416,-2 -0.2941176470588236)),((-2 -0.05882352941176472,2 -0.05882352941176472,2 0.05882352941176472,-2 0.05882352941176472,-2 -0.05882352941176472)),((-2 0.17647058823529416,2 0.17647058823529416,2 0.29411764705882365,-2 0.2941176470588236,-2 0.17647058823529416)),((-2 0.4117647058823528,2 0.41176470588235287,2 0.5294117647058821,-2 0.5294117647058822,-2 0.4117647058823528)),((-2 0.6470588235294117,2 0.6470588235294118,2 0.7647058823529411,-2 0.7647058823529411,-2 0.6470588235294117)),((-2 0.8823529411764706,2 0.8823529411764707,2 1.0000000000000002,-2 1,-2 0.8823529411764706)))";

    let mp1 = MultiPolygon::<f64>::try_from_wkt_str(wkt1)?;
    let mp2 = MultiPolygon::<f64>::try_from_wkt_str(wkt2)?;

    for p1 in mp1.0.iter() {
        let p1 = MultiPolygon::from(p1.clone());
        for p2 in mp2.0.iter() {
            let p2 = MultiPolygon::from(p2.clone());
            let result = catch_unwind(|| -> Result<()> {
                check_sweep(&p1.wkt_string(), &p2.wkt_string(), OpType::Union)?;
                Ok(())
            });
            if let Err(ee) = result {
                error!("p1: {wkt}", wkt = p1.wkt_string());
                error!("p2: {wkt}", wkt = p2.wkt_string());
                resume_unwind(ee);
            }
        }
    }
    Ok(())
}
#[test]
fn test_complex_rects1() -> Result<()> {
    let wkt1 = "MULTIPOLYGON(((-1 -2,-1.0000000000000002 2,-0.8823529411764707 2,-0.8823529411764706 -2,-1 -2)))";
    let wkt2 = "MULTIPOLYGON(((-2 -1,2 -1.0000000000000002,2 -0.8823529411764707,-2 -0.8823529411764706,-2 -1)))";
    check_sweep(wkt1, wkt2, OpType::Union)?;
    Ok(())
}
