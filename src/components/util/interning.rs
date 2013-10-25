#[feature(globs)];

extern mod extra;
use std::{vec,ptr,cast,str};
use std::ascii::StrAsciiExt;
use std::to_bytes::Cb;
use std::cmp::Equiv;

#[deriving(Clone)]
struct Rawptr {priv p: *BucketNode}

struct InterningStr {
    priv buckets: ~[Option<~BucketNode>],
    priv lens: ~[uint],
}

#[deriving(Clone)]
struct BucketNode {
    s: ~str,
    hash: u32,
    next: Option<~BucketNode>,
}

struct MutBucketNodeIterator<'self> {
    priv interningStr: &'self mut InterningStr,
    priv cur: Rawptr,
    priv nelem: uint,
}

#[deriving(Clone)]
pub struct IntString {
    priv s: Rawptr,
    priv lowercases: Rawptr,
}

static mut interning: Option<InterningStr> = None;

pub fn init() {
    unsafe {
        match interning {
            None => {
                interning = Some(InterningStr::new());
            }
            Some(_) => ()
        }
    }
}

pub fn intern_string(s: &str) -> IntString {
    unsafe {
        match interning {
            Some(ref mut n) => {
                n.intern_string(s)
            }
            None => fail!("Interning: init() required")
        }
    }
}

pub fn to_rust_string(s: &IntString) -> ~str {
    s.to_str()
}

pub fn to_rust_string_slice<'a>(s: &'a IntString) -> &'a str {
    s.to_str_slice()
}

impl InterningStr {
    fn new() -> InterningStr {
        let size = 1021u;
        InterningStr {
            buckets: vec::from_elem(size, None),
            lens: vec::from_elem(size, 0u),
        }
    }

    fn universal_hash(s: &str) -> u32 {
        let mut hash: u32 = 0;
        for c in s.byte_iter() {
            hash += SEEDS_A[c] * c as u32 + SEEDS_B[c]
        }
        return hash % 1511;
    }

    fn intern_string(&mut self, s: &str) -> IntString {
        let ptr = self.intern_string_internal(s);
        let lowercases_ptr = self.intern_string_internal(s.to_ascii_lower());
        IntString {
            s: ptr,
            lowercases: lowercases_ptr,
        }
    }

    fn intern_string_internal(&mut self, s: &str) -> Rawptr {
        let h = InterningStr::universal_hash(s);
        let i = (h as uint) % self.buckets.len();

        match self.buckets[i] {
            Some(_) => {
                return self.push_back_node(s, h, i);
            }
            None => {
                self.buckets[i] = Some(~BucketNode {
                    s: s.to_str(),
                    hash: h,
                    next: None,
                });
                self.lens[i]=1;

                let ptr = match self.buckets[i] {
                    Some(ref mut node) => {
                        Rawptr::some(*node)
                    }
                    None => fail!("Interning: internal logic error")
                };
                return ptr;
            }
        }
    }

    #[inline]
    fn mut_iter<'a>(&'a mut self, index: uint) -> MutBucketNodeIterator<'a> {
        let cur = match self.buckets[index] {
            Some(ref mut node) => Rawptr::some(*node),
            None => Rawptr::none(),
        };
        MutBucketNodeIterator {
            cur: cur,
            nelem: self.lens[index],
            interningStr: self,
        }
    }

    #[inline]
    fn push_back_node(&mut self, s: &str, h: u32, i: uint) -> Rawptr {
        let mut itr = self.mut_iter(i);
        loop {
            match itr.peek() {
                Some(node) => {
                    if (node.hash == h) && (node.s.len() == s.len()) {
                        if str::eq_slice(node.s, s) {
                            return Rawptr::some(node);
                        }
                    }
                }
                None => fail!("Interning: internal logic error"),
            }

            if itr.has_next() {
                itr.next();
            } else {
                break;
            }
        }
        itr.insert_next(s, h, i)
    }
}

impl<'self> Iterator<&'self mut BucketNode> for MutBucketNodeIterator<'self> {
    #[inline]
    fn next(&mut self) -> Option<&'self mut BucketNode> {
        if self.nelem == 0 {
            return None;
        }
        do self.cur.resolve().map |cur| {
            self.nelem -= 1;
            self.cur = match cur.next {
                Some(ref mut next_node) => {
                    Rawptr::some(*next_node)
                }
                None => Rawptr::none(),
            };
            cur
        }
    }
}

impl<'self> MutBucketNodeIterator<'self> {
    #[inline]
    fn has_next(&self) -> bool {
        if !self.is_last() {
            return true;
        } else {
            return false;
        }
    }

    #[inline]
    fn is_last(&self) -> bool {
        if self.nelem == 1 {
            return true;
        } else {
            return false;
        }
    }

    #[inline]
    fn peek<'a>(&'a mut self) -> Option<&'a mut BucketNode> {
        if self.nelem == 0 {
            return None;
        } else {
            return self.cur.resolve().map(|node| node);
        }
    }

    #[inline]
    fn insert_next(&mut self, s: &str, h: u32, i: uint) -> Rawptr {
        match self.cur.resolve() {
            Some(node) => {
                node.next = Some(~BucketNode {
                    s: s.to_str(),
                    hash: h,
                    next: None,
                });
                self.interningStr.lens[i]+=1;

                let ptr = match node.next {
                    Some(ref mut new_node) => {
                        Rawptr::some(*new_node)
                    }
                    None => fail!("Interning: internal logic error")
                };
                return ptr;
            }
            None => fail!("Interning: internal logic error")
        }
    }
}

impl Rawptr {
    fn none() -> Rawptr {
        Rawptr {p: ptr::null()}
    }

    fn some(n: &mut BucketNode) -> Rawptr {
        Rawptr {p: ptr::to_unsafe_ptr(n)}
    }

    fn resolve(&mut self) -> Option<&mut BucketNode> {
        if self.p.is_null() {
            None
        } else {
            Some(unsafe {cast::transmute(self.p)})
        }
    }

    fn resolve_immut(&self) -> Option<& BucketNode> {
        unsafe {self.p.to_option()}
    }
}

impl Eq for IntString {
    #[inline]
    fn eq(&self, other: &IntString) -> bool {
        self.s.p == other.s.p
    }
}

impl ToStr for IntString {
    #[inline]
    fn to_str(&self) -> ~str {
        self.to_str_slice().to_str()
    }
}

impl IterBytes for IntString {
    #[inline]
    fn iter_bytes(&self, lsb0: bool, f: Cb) -> bool {
        self.to_str_slice().iter_bytes(lsb0, f)
    }
}

impl Equiv<IntString> for IntString {
    #[inline]
    fn equiv(&self, other: &IntString) -> bool {
        self.eq_ignore_ascii_case(other)
    }
}

impl<'self> IntString {
    pub fn to_str_slice(&'self self) -> &'self str {
        match self.s.resolve_immut() {
            Some(ref node) => {
                let s: &'self str = node.s;
                s
            }
            None => fail!("Interning: internal logic error")
        }
    }

    #[inline]
    pub fn eq_ignore_ascii_case(&self, other: &IntString) -> bool {
        self.lowercases.p == other.lowercases.p
    }

    #[inline]
    pub fn to_ascii_lower(&self) -> &'self str {
        match self.lowercases.resolve_immut() {
            Some(ref node) => {
                return node.s.as_slice();
            }
            None => fail!("Interning: internal logic error")
        }
    }
}

static SEEDS_A: &'static [u32] = &[
    991, 261, 363, 1138, 78, 1036, 1455, 782,
    835, 1186, 1108, 391, 1503, 144, 1322, 33,
    648, 903, 429, 57, 89, 1501, 1000, 927,
    362, 1227, 1109, 1406, 40, 133, 222, 366,
    269, 18, 1450, 2, 1118, 748, 113, 98,
    517, 1065, 479, 1183, 1111, 798, 69, 113,
    1134, 969, 1159, 819, 863, 388, 616, 179,
    970, 11, 699, 188, 395, 1325, 834, 846,
    1011, 39, 434, 424, 288, 67, 307, 1285,
    1415, 1401, 1233, 1459, 635, 425, 1107, 11,
    1127, 75, 205, 522, 1003, 746, 1506, 985,
    163, 534, 559, 693, 188, 1160, 270, 1136,
    586, 890, 1276, 1065, 134, 1441, 505, 951,
    1461, 1427, 28, 759, 1013, 421, 1484, 222,
    1466, 120, 861, 823, 550, 650, 347, 1450,
    1192, 397, 1449, 871, 812, 340, 1328, 579,
    1086, 964, 395, 1267, 1195, 531, 1097, 667,
    531, 1165, 593, 481, 883, 827, 549, 646,
    671, 112, 904, 1148, 173, 627, 1217, 1211,
    142, 547, 855, 43, 249, 660, 1121, 1064,
    1227, 499, 1212, 640, 1329, 1294, 1221, 521,
    130, 710, 1287, 1007, 1411, 33, 23, 614,
    431, 530, 17, 580, 150, 267, 1447, 1016,
    946, 1234, 841, 472, 1072, 673, 281, 976,
    770, 618, 867, 858, 1397, 17, 878, 1084,
    893, 1495, 801, 388, 58, 814, 924, 745,
    197, 1390, 1454, 1125, 853, 171, 614, 1433,
    950, 1052, 758, 1034, 370, 948, 1343, 997,
    119, 492, 1350, 1049, 1305, 1068, 1021, 974,
    50, 475, 81, 1421, 1478, 1445, 728, 22,
    1134, 1455, 795, 56, 103, 1110, 1140, 1481,
    705, 1210, 925, 628, 229, 920, 691, 426,
];

static SEEDS_B: &'static [u32] = &[
    1220, 248, 477, 424, 76, 791, 1058, 738,
    248, 408, 864, 878, 840, 1167, 708, 549,
    1136, 779, 515, 337, 1416, 926, 1294, 560,
    424, 382, 1276, 1062, 457, 155, 528, 389,
    180, 162, 334, 1121, 78, 151, 778, 1428,
    1260, 1130, 1236, 242, 323, 753, 1419, 1107,
    244, 413, 289, 247, 171, 558, 205, 1510,
    1365, 1023, 1108, 506, 473, 405, 271, 1141,
    1093, 1292, 875, 260, 1016, 1144, 298, 228,
    1054, 822, 1425, 694, 674, 154, 230, 538,
    925, 652, 157, 1042, 354, 572, 1498, 73,
    328, 939, 1075, 1374, 553, 1466, 1411, 915,
    1109, 51, 176, 574, 595, 1192, 1510, 855,
    1044, 1116, 1249, 941, 202, 1235, 607, 944,
    1357, 145, 948, 10, 285, 1338, 866, 711,
    575, 238, 1477, 68, 13, 949, 143, 418,
    1223, 275, 64, 142, 1206, 26, 1190, 1044,
    593, 231, 1327, 395, 1120, 1314, 1279, 478,
    962, 184, 705, 278, 134, 1277, 875, 1211,
    196, 664, 560, 1334, 1006, 1084, 96, 1103,
    394, 4, 507, 638, 199, 1005, 136, 583,
    1245, 936, 394, 719, 368, 997, 1268, 717,
    752, 822, 464, 1315, 1342, 1493, 1186, 4,
    733, 301, 539, 842, 710, 588, 539, 1216,
    635, 784, 535, 0, 124, 760, 1346, 847,
    775, 1261, 539, 251, 1260, 723, 986, 280,
    939, 85, 765, 185, 140, 744, 1030, 606,
    489, 969, 1279, 1357, 345, 655, 546, 144,
    953, 1037, 731, 536, 1375, 452, 991, 1045,
    1119, 371, 1048, 556, 984, 1399, 179, 1048,
    1055, 71, 294, 1479, 316, 1277, 1303, 1205,
    518, 1207, 44, 236, 33, 503, 849, 931,
];

mod seeds {
    use std::rand;
    use std::rand::Rng;

    fn gen_random_numbers(max: u32) {
        let mut rng = rand::rng();
        if rng.gen() {
            for _i in range(0, 32) {
                for _j in range(0, 8) {
                    print!("{}, ", rng.gen::<u32>() % max);
                }
                print!("{}", "\n");
            }
        }
    }
}

#[test]
fn interning_test() {
    init();
    let s1 = intern_string("test");
    let s2 = intern_string("test");
    let s3 = intern_string("toast");

    assert!(s1.eq(&s2));
    assert!(!s1.eq(&s3));
    assert!(str::eq_slice(s1.to_str(), s2.to_str()));
    assert!(!str::eq_slice(s1.to_str(), s3.to_str()));
    assert!(str::eq_slice(s1.to_str_slice(), s2.to_str_slice()));
    assert!(!str::eq_slice(s1.to_str_slice(), s3.to_str_slice()));
}

#[test]
fn smoke_test() {
    let mut buckets = ~InterningStr::new();
    let s1 = buckets.intern_string("test");
    let s2 = buckets.intern_string("test");
    let s3 = buckets.intern_string("toast");
    let s4 = buckets.intern_string("TOAST");
    let s5 = buckets.intern_string("AAa");
    let s6 = buckets.intern_string("aAA");

    assert!(s1.eq(&s2));
    assert!(!s1.eq(&s3));
    assert!(str::eq(&s1.to_str(), &s2.to_str()));
    assert!(!str::eq(&s1.to_str(), &s3.to_str()));
    assert!(str::eq_slice(s1.to_str_slice(), s2.to_str_slice()));
    assert!(!str::eq_slice(s1.to_str_slice(), s3.to_str_slice()));
    assert!(!s3.eq(&s4));
    assert!(s3.eq_ignore_ascii_case(&s4));
    assert!(!s5.eq(&s6));
    assert!(s5.eq_ignore_ascii_case(&s6));
}

#[cfg(test)]
mod bench {
    use super::*;
    use std::hashmap::HashSet;

    #[bench]
    fn hashmap() {
        let words = words();
        let mut hashmap = HashSet::new();
        for word in words.iter() {
            hashmap.insert(word);
        }
    }

    #[bench]
    fn interning() {
        let words = words();
        let mut buckets = ~InterningStr::new();
        for word in words.iter() {
            buckets.intern_string(*word);
        }
    }

    fn words() -> ~[~str] {
        let words = ~[~"<html>",
                      ~"<head>",
                      ~"<style", ~"type=\"text/css\">",
                      ~"div", ~"{", ~"font-size:30px;", ~"}",
                      ~"/*", ~"NOT_IMPLEMENTED",
                      ~"div:last-child", ~"{", ~"color:orange;", ~"font-size:70px;", ~"}",
                      ~"a:link", ~"{color:orange;", ~"font-size:70px;}",
                      ~"span+div", ~"{", ~"color:blue;", ~"}",
                      ~"span:last-child", ~"{", ~"font-style:italic;", ~"}",
                      ~"*/",
                      ~"h2", ~"span", ~"{", ~"color:red;", ~"}",
                      ~"div,", ~"span,", ~"p", ~"{", ~"font-family:\"Georgia\";", ~"}",
                      ~"div,", ~"span", ~"{", ~"font-size:40px;", ~"}",
                      ~"#left", ~"{", ~"text-align:left;", ~"}",
                      ~".center", ~"{", ~"text-align:center;", ~"}",
                      ~".right", ~"{", ~"text-align:right;", ~"}",
                      ~"div.bgorange", ~"{", ~"background-color:orange;", ~"}",
                      ~".bggreen{", ~"background-color:#11FF22;", ~"}",
                      ~".bggreen", ~"{", ~"background-color:green;}",
                      ~".bgblue", ~"{", ~"background-color:blue;", ~"}",
                      ~".bgyellow", ~"{", ~"background-color:yellow;", ~"}",
                      ~".red",~"{", ~"color:red;", ~"}",
                      ~".red.red2", ~"{", ~"font-size:40px;", ~"}",
                      ~".blue", ~"{", ~"color:blue;", ~"}",
                      ~"#gray", ~"{", ~"color:gray;", ~"}",
                      ~".green", ~"{", ~"color:green;", ~"}",
                      ~".yellow", ~"{", ~"color:yellow;", ~"}",
                      ~"div.white", ~"{", ~"color:white;", ~"}",
                      ~"span.white", ~"{", ~"color:white;", ~"font-size:20px;", ~"}",
                      ~"span.orange", ~"{", ~"color:orange;", ~"}",
                      ~"div>.orange", ~"{", ~"color:orange;", ~"font-size:30px;", ~"}",
                      ~".italic", ~"{", ~"font-style:italic;", ~"}",
                      ~"span.times", ~"{", ~"font-family:\"Times", ~"New", ~"Roman;\"", ~"}",
                      ~".geor", ~"{", ~"font-family:\"Georgia;\"", ~"}",
                      ~"#arial", ~"{", ~"font-family:\"Arial;\"", ~"}",
                      ~".ver", ~"{", ~"font-family:\"Verdana;\"", ~"}",
                      ~".under", ~"{", ~"text-decoration:underline;", ~"}",
                      ~".size30", ~"{", ~"font-size:30px;", ~"}",
                      ~".size45", ~"{", ~"font-size:45px;", ~"}",
                      ~".size60", ~"{", ~"font-size:60px;", ~"}",
                      ~".size2em", ~"{", ~"font-size:2em;", ~"}",
                      ~"h2", ~"div", ~".size50p", ~"{", ~"font-size:50%;", ~"}",
                      ~"#border_solid_5", ~"{", ~"border-style:solid;", ~"border-width:5px;", ~"}",
                      ~".bcolor", ~"{", ~"border-color:red;", ~"}",
                      ~"div.bcolor", ~"{", ~"border-style:solid;", ~"border-width:3px;", ~"font-size:20px;", ~"}",
                      ~"</style>",
                      ~"</head>",
                      ~"<body>",
                      ~"<div", ~"class=\"bggreen\">",
                      ~"<div", ~"id=\"left\">",
                      ~"CSS", ~"text", ~"align", ~"test",
                      ~"</div>",
                      ~"<div", ~"class=\"center", ~"bgorange\">",
                      ~"CSS", ~"text", ~"align", ~"test",
                      ~"</div>",
                      ~"<div", ~"class=\"right", ~"under\">",
                      ~"CSS", ~"text", ~"align", ~"test", ~"</div>",
                      ~"</div>",
                      ~"<div>",
                      ~"<span", ~"class=\"red", ~"red2\">This</span>",
                      ~"<span", ~"class=\"blue\">", ~"is</span>",
                      ~"<span", ~"id=\"gray\">CSS</span>",
                      ~"<span", ~"class=\"green\">Text",
                      ~"</span>",
                      ~"<span", ~"class=\"orange\">Color</span>",
                      ~"<span>test</span>",
                      ~"</div>",
                      ~"<h2", ~"class=\"bggreen", ~"center\">",
                      ~"CSS", ~"Font", ~"test",
                      ~"<hr/>",
                      ~"<span", ~"class=\"under\">",
                      ~"underlined",
                      ~"text</span>",
                      ~"<span", ~"class=\"italic", ~"orange\">",
                      ~"italic",
                      ~"</span>",
                      ~"<span", ~"class=\"times\">",
                      ~"Times",
                      ~"New",
                      ~"Roman",
                      ~"</span>",
                      ~"<span", ~"class=\"ver",
                      ~"blue\">",
                      ~"Verdana",
                      ~"</span>",
                      ~"<span", ~"id=\"arial\"",
                      ~"class=\"orange\">",
                      ~"Arial",
                      ~"</span>",
                      ~"<span", ~"class=\"white\"",
                      ~"style=\"font-family:Courier",
                      ~"New\">",
                      ~"Courier", ~"New",
                      ~"</span>",
                      ~"<span", ~"class=\"geor",
                      ~"red",
                      ~"bgyellow\">",
                      ~"Georgia</span>",
                      ~"<span", ~"style=\"font-family:Lucida", ~"Console\"",
                      ~"class=\"size45",
                      ~"yellow\">",
                      ~"Lucida", ~"Console",
                      ~"</span>",
                      ~"<div", ~"class=\"white",
                      ~"bgblue\">",
                      ~"<span", ~"class=\"size30\">",
                      ~"30px",
                      ~"</span>",
                      ~"<span", ~"class=\"size45",
                      ~"yellow\">",
                      ~"45px",
                      ~"</span>",
                      ~"<span", ~"class=\"size60\">",
                      ~"60px",
                      ~"</span>",
                      ~"basesize-30px",
                      ~"<span", ~"class=\"size2em",
                      ~"bgorange\">",
                      ~"2em",
                      ~"</span>",
                      ~"<span", ~"class=\"size50p",
                      ~"under",
                      ~"bggreen\">",
                      ~"50%",
                      ~"</span>",
                      ~"</div>",
                      ~"</h2>",
                      ~"<div>",
                      ~"<div", ~"class=\"bcolor", ~"bgorange\">",
                      ~"border",
                      ~"test",
                      ~"</div>",
                      ~"<div", ~"id=\"border_solid_5\"",
                      ~"class=\"bgyellow",
                      ~"blue", ~"ver", ~"right\"", ~">",
                      ~"border",
                      ~"<span", ~"class=\"bggreen\">green</span>",
                      ~"</div>",
                      ~"<p", ~"id=\"border_solid_5\"",
                      ~"class=\"bcolor\">",
                      ~"border",
                      ~"test",
                      ~"<span", ~"class=\"italic",
                      ~"bgyellow",
                      ~"red\">abcde</span>",
                      ~"</p>",
                      ~"</div>",
                      ~"</body>",
                      ~"</html>"];
        words
    }
}
