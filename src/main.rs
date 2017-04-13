extern crate xmltree;
extern crate time;
extern crate zip;

use std::error::Error;
use std::fs::File;
use std::vec::Vec;
use std::process;
use std::env;
use xmltree::Element;
use zip::read::ZipArchive;
use time::{Tm, Duration, strptime};

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() < 2 {
        println!("Must provide a path to gpx file!");
        process::exit(1);
    }
    let ref path = args[1];
    let file = match File::open(path) {
        Err(why) => panic!("couldn't open file {}: {}", path, why.description()),
        Ok(file) => file,
    };
    let mut zip = ZipArchive::new(file).unwrap();
    let mut total_rides = 0;
    let mut total_duration = 0;
    let mut total_elevation = 0.0;
    let mut total_distance = 0.0;
    for i in 0..zip.len(){
        let zipfile = zip.by_index(i).unwrap();
        let gpx = Element::parse(zipfile).unwrap();
        let title = get_title(&gpx);
        println!("{}", title);
        let points = get_points(&gpx);
        println!("Number of samples: {}", points.len());
        println!("Start time: {}", points[0].time.asctime());
        let duration = get_duration(&points).num_seconds();
        total_duration += duration;
        println!("Total time: {} seconds", duration);
        let elevation = get_elevation(&points);
        total_elevation += elevation;
        println!("Total elevation: {} meters", elevation);
        let distance = get_distance(&points);
        total_distance += distance;
        println!("Total distance: {} meters", distance);
        total_rides+=1;
    }
    println!(
        "{{\"rides\": {}, \"duration\": {}, \"elevation\": {}, \"distance\": {}}}",
        total_rides, total_duration, total_elevation, total_distance
    )
}

struct Point {
    ele: f32,
    time: Tm,
    lat: f32,
    lon: f32,
}


fn distance(a: &Point, b: &Point) -> f32 {
    // Haversine formula
    let r = 6371e3;
    let lat1_rad = a.lat.to_radians();
    let lat2_rad = b.lat.to_radians();
    let lat_dif = (b.lat - a.lat).to_radians();
    let lon_dif = (b.lon - a.lon).to_radians();
    let a = (lat_dif / 2.0).sin() * (lat_dif / 2.0).sin() +
            lat1_rad.cos() * lat2_rad.cos() * (lon_dif / 2.0).sin() * (lon_dif / 2.0).sin();
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    return r * c;
}

fn get_title(gpx_element: &xmltree::Element) -> &str {
    let name_elem = gpx_element
        .get_child("trk")
        .expect("not a valid gpx")
        .get_child("name")
        .expect("not a valid gpx");
    let title = match name_elem.text {
        Some(ref x) => x,
        None => "Unnamed Activity",
    };
    return title;
}

fn get_points(gpx_element: &xmltree::Element) -> Vec<Point> {
    let mut points = vec![];
    let ref children = gpx_element
        .get_child("trk")
        .expect("not a vlaid gpx")
        .get_child("trkseg")
        .expect("not a valid gpx")
        .children;
    for child in children {
        let ele_elem = child.get_child("ele").unwrap();
        let ele = match ele_elem.text {
            Some(ref x) => x.parse().unwrap(),
            None => 0.0,
        };
        let time_elem = child.get_child("time").unwrap();
        let time = match time_elem.text {
            Some(ref x) => strptime(x, "%Y-%m-%dT%H:%M:%S"),
            None => panic!("bad time element"),
        };
        points.push(Point {
                        ele: ele,
                        time: time.unwrap(),
                        lat: child.attributes.get("lat").unwrap().parse().unwrap(),
                        lon: child.attributes.get("lon").unwrap().parse().unwrap(),
                    });

    }
    return points;
}


fn get_duration(points: &Vec<Point>) -> Duration {
    points[points.len() - 1].time - points[0].time
}

fn get_elevation(points: &Vec<Point>) -> f32 {
    let mut ele = 0.0;
    for (idx, point) in points.iter().enumerate() {
        if idx == 0 {
            ele = point.ele;
        } else {
            if point.ele > points[idx - 1].ele {
                ele += point.ele - points[idx - 1].ele;
            }
        }
    }
    return ele;
}

fn get_distance(points: &Vec<Point>) -> f32 {
    let mut d = 0.0;
    for (idx, point) in points.iter().enumerate() {
        if idx == 0 {
            continue;
        }
        d = d + distance(&point, &points[idx - 1]);
    }
    return d;
}
