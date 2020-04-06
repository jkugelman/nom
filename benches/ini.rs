#[macro_use]
extern crate nom;
extern crate criterion;
extern crate jemallocator;

#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

use criterion::*;

use nom::{
  IResult,
  combinator::map_res,
  sequence::delimited,
  bytes::complete::take_while,
  character::complete::{alphanumeric1 as alphanumeric, multispace1 as multispace, space1 as space, char}
};
use std::str;
use std::collections::HashMap;

fn category(i: &[u8]) -> IResult<&[u8], &str> {
  map_res(delimited(char('['), take_while(|c| c != b']'), char(']')), str::from_utf8)(i)
}

named!(key_value    <&[u8],(&str,&str)>,
  do_parse!(
     key: map_res!(alphanumeric, str::from_utf8)
  >>      opt!(space)
  >>      char!('=')
  >>      opt!(space)
  >> val: map_res!(
           take_while!(call!(|c| c != '\n' as u8 && c != ';' as u8)),
           str::from_utf8
         )
  >>      opt!(pair!(char!(';'), take_while!(call!(|c| c != '\n' as u8))))
  >>      (key, val)
  )
);

named!(keys_and_values<&[u8], HashMap<&str, &str> >,
  map!(
    many0!(terminated!(key_value, opt!(multispace))),
    |vec: Vec<_>| vec.into_iter().collect()
  )
);

named!(category_and_keys<&[u8],(&str,HashMap<&str,&str>)>,
  do_parse!(
    category: category         >>
              opt!(multispace) >>
    keys: keys_and_values      >>
    (category, keys)
  )
);

named!(categories<&[u8], HashMap<&str, HashMap<&str,&str> > >,
  map!(
    many0!(
      separated_pair!(
        category,
        opt!(multispace),
        map!(
          many0!(terminated!(key_value, opt!(multispace))),
          |vec: Vec<_>| vec.into_iter().collect()
        )
      )
    ),
    |vec: Vec<_>| vec.into_iter().collect()
  )
);

/*
#[test]
fn parse_category_test() {
  let ini_file = &b"[category]

parameter=value
key = value2"[..];

  let ini_without_category = &b"parameter=value
key = value2"[..];

  let res = category(ini_file);
  println!("{:?}", res);
  match res {
    IResult::Done(i, o) => println!("i: {:?} | o: {:?}", str::from_utf8(i), o),
    _ => println!("error")
  }

  assert_eq!(res, IResult::Done(ini_without_category, "category"));
}

#[test]
fn parse_key_value_test() {
  let ini_file = &b"parameter=value
key = value2"[..];

  let ini_without_key_value = &b"key = value2"[..];

  let res = key_value(ini_file);
  println!("{:?}", res);
  match res {
    IResult::Done(i, (o1, o2)) => println!("i: {:?} | o: ({:?},{:?})", str::from_utf8(i), o1, o2),
    _ => println!("error")
  }

  assert_eq!(res, IResult::Done(ini_without_key_value, ("parameter", "value")));
}


#[test]
fn parse_key_value_with_space_test() {
  let ini_file = &b"parameter = value
key = value2"[..];

  let ini_without_key_value = &b"key = value2"[..];

  let res = key_value(ini_file);
  println!("{:?}", res);
  match res {
    IResult::Done(i, (o1, o2)) => println!("i: {:?} | o: ({:?},{:?})", str::from_utf8(i), o1, o2),
    _ => println!("error")
  }

  assert_eq!(res, IResult::Done(ini_without_key_value, ("parameter", "value")));
}

#[test]
fn parse_key_value_with_comment_test() {
  let ini_file = &b"parameter=value;abc
key = value2"[..];

  let ini_without_key_value = &b"key = value2"[..];

  let res = key_value(ini_file);
  println!("{:?}", res);
  match res {
    IResult::Done(i, (o1, o2)) => println!("i: {:?} | o: ({:?},{:?})", str::from_utf8(i), o1, o2),
    _ => println!("error")
  }

  assert_eq!(res, IResult::Done(ini_without_key_value, ("parameter", "value")));
}

#[test]
fn parse_multiple_keys_and_values_test() {
  let ini_file = &b"parameter=value;abc

key = value2

[category]"[..];

  let ini_without_key_value = &b"[category]"[..];

  let res = keys_and_values(ini_file);
  println!("{:?}", res);
  match res {
    IResult::Done(i, ref o) => println!("i: {:?} | o: {:?}", str::from_utf8(i), o),
    _ => println!("error")
  }

  let mut expected: HashMap<&str, &str> = HashMap::new();
  expected.insert("parameter", "value");
  expected.insert("key", "value2");
  assert_eq!(res, IResult::Done(ini_without_key_value, expected));
}

#[test]
fn parse_category_then_multiple_keys_and_values_test() {
  //FIXME: there can be an empty line or a comment line after a category
  let ini_file = &b"[abcd]
parameter=value;abc

key = value2

[category]"[..];

  let ini_after_parser = &b"[category]"[..];

  let res = category_and_keys(ini_file);
  println!("{:?}", res);
  match res {
    IResult::Done(i, ref o) => println!("i: {:?} | o: {:?}", str::from_utf8(i), o),
    _ => println!("error")
  }

  let mut expected_h: HashMap<&str, &str> = HashMap::new();
  expected_h.insert("parameter", "value");
  expected_h.insert("key", "value2");
  assert_eq!(res, IResult::Done(ini_after_parser, ("abcd", expected_h)));
}

#[test]
fn parse_multiple_categories_test() {
  let ini_file = &b"[abcd]

parameter=value;abc

key = value2

[category]
parameter3=value3
key4 = value4
\0"[..];

  let ini_after_parser = &b"\0"[..];

  let res = categories(ini_file);
  //println!("{:?}", res);
  match res {
    IResult::Done(i, ref o) => println!("i: {:?} | o: {:?}", str::from_utf8(i), o),
    _ => println!("error")
  }

  let mut expected_1: HashMap<&str, &str> = HashMap::new();
  expected_1.insert("parameter", "value");
  expected_1.insert("key", "value2");
  let mut expected_2: HashMap<&str, &str> = HashMap::new();
  expected_2.insert("parameter3", "value3");
  expected_2.insert("key4", "value4");
  let mut expected_h: HashMap<&str, HashMap<&str, &str>> = HashMap::new();
  expected_h.insert("abcd",     expected_1);
  expected_h.insert("category", expected_2);
  assert_eq!(res, IResult::Done(ini_after_parser, expected_h));
}
*/

fn bench_ini(c: &mut Criterion) {
  let str = "[owner]
name=John Doe
organization=Acme Widgets Inc.

[database]
server=192.0.2.62
port=143
file=payroll.dat
\0";

  c.bench(
    "bench ini",
    Benchmark::new(
      "parse",
      move |b| {
        b.iter(|| categories(str.as_bytes()).unwrap());
      },
    ).throughput(Throughput::Bytes(str.len() as u64)),
  );
}

fn bench_ini_keys_and_values(c: &mut Criterion) {
  let str = "server=192.0.2.62
port=143
file=payroll.dat
\0";

  named!(acc<Vec<(&str, &str)>>, many0!(key_value));

  c.bench(
    "bench ini keys and values",
    Benchmark::new(
      "parse",
      move |b| {
        b.iter(|| acc(str.as_bytes()).unwrap());
      },
    ).throughput(Throughput::Bytes(str.len() as u64)),
  );
}

fn bench_ini_key_value(c: &mut Criterion) {
  let str = "server=192.0.2.62\n";

  c.bench(
    "bench ini key value",
    Benchmark::new(
      "parse",
      move |b| {
        b.iter(|| key_value(str.as_bytes()).unwrap());
      },
    ).throughput(Throughput::Bytes(str.len() as u64)),
  );
}

criterion_group!(benches, bench_ini, bench_ini_keys_and_values, bench_ini_key_value);
criterion_main!(benches);
