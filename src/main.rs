use clap::Parser;
use gdal::Dataset;
use gdal::GeoTransformEx;
use geo::{point, HaversineDistance};
use geojson::{FeatureCollection, GeoJson, Value};
use poloto::build;
use tagu::prelude::*;
use utm::to_utm_wgs84;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    filename: String,

    #[arg(short, long)]
    geojson: String,
}

fn to_svg(data: &Vec<(f64, f64, f64)>) {
    let theme = poloto::render::Theme::dark().append(tagu::build::raw(
        ".poloto0.poloto_line{fill:hsla(200, 100%, 50%, 0.2); stroke:hsl(200, 100%, 50%);}",
    ));

    poloto::frame()
        .with_tick_lines([true, true])
        .build()
        .data(poloto::plots!(
            poloto::build::origin(),
            build::plot("DEM").line(data.iter().map(|(dist, dem, _)| (dist, dem))),
            build::plot("GPS").line(data.iter().map(|(dist, _, gps)| (dist, gps)))
        ))
        .build_and_label(("Elevation", "Distance (meters)", "Altitude (meters)"))
        .append_to(poloto::header().append(theme))
        .render_stdout();
}

fn main() {
    let args = Args::parse();
    let ds = Dataset::open(&args.filename).unwrap();
    eprintln!(
        "Loading file {} {} {} bands: {}",
        args.filename,
        ds.driver().long_name(),
        ds.spatial_ref().unwrap().name().unwrap(),
        ds.raster_count()
    );
    let band = ds.rasterband(1).unwrap();

    let transform = ds.geo_transform().unwrap();
    let inv_transform = transform.invert().unwrap();

    let geojson_str: String = std::fs::read_to_string(args.geojson).unwrap();
    let geojson = geojson_str.parse::<GeoJson>().unwrap();

    let features = FeatureCollection::try_from(geojson).unwrap();

    let mut distance: f64 = 0.0;
    let mut total: Vec<(f64, f64, f64)> = vec![];
    for feature in features {
        if let Some(geom) = feature.geometry {
            let coordinates = match geom.value {
                Value::LineString(line_string) => line_string,
                _ => panic!("Expected a LineString"),
            };

            let mut result: Vec<(f64, f64, f64)> = coordinates
                .iter()
                .enumerate()
                .map(|(index, coord)| {
                    let (lng, lat, elevation) = (coord[0], coord[1], coord[2]);
                    distance += if index > 0 {
                        let p1 = point!(x: coord[0], y: coord[1]);
                        let prev = &coordinates[index - 1];
                        let p0 = point!(x: prev[0], y: prev[1]);
                        p1.haversine_distance(&p0)
                    } else {
                        0.0
                    };

                    let (north, east, _) = to_utm_wgs84(lat, lng, 33);
                    let (tx, ty) = inv_transform.apply(east, north);
                    let buf = band
                        .read_as::<f64>((tx as isize, ty as isize), (1, 1), (1, 1), None)
                        .unwrap();
                    (distance.round(), buf.data[0].round(), elevation)
                })
                .collect();
            total.append(&mut result);
        }
    }
    to_svg(&total);
}
