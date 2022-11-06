extern crate libutils;

/// Test the Path Iterator
#[test]
pub fn test_path_iterator() {
    use libutils::paths::OwnedPath;

    let path0 = OwnedPath::new("/usr/bin/ls");
    let path1 = OwnedPath::new("bin/ls");
    let path2 = OwnedPath::new("/");
    let path3 = OwnedPath::new("./../../home/");

    assert_eq!(path0.iter().collect::<Vec<_>>(), vec!["usr", "bin", "ls"]);
    assert_eq!(path1.iter().collect::<Vec<_>>(), vec!["bin", "ls"]);
    assert_eq!(path2.iter().collect::<Vec<_>>(), Vec::<&str>::new());
    assert_eq!(
        path3.iter().collect::<Vec<_>>(),
        vec![".", "..", "..", "home"]
    );

    assert_eq!(
        (&path0).iter().collect::<Vec<_>>(),
        vec!["usr", "bin", "ls"]
    );
    assert_eq!((&path1).iter().collect::<Vec<_>>(), vec!["bin", "ls"]);
    assert_eq!((&path2).iter().collect::<Vec<_>>(), Vec::<&str>::new());
    assert_eq!(
        (&path3).iter().collect::<Vec<_>>(),
        vec![".", "..", "..", "home"]
    );
}

/// Test the Path Canonicalization
#[test]
pub fn test_path_canonicalization() {
    use libutils::paths::OwnedPath;

    let mut path0 = OwnedPath::new("/usr/bin/ls");
    let mut path1 = OwnedPath::new("bin/ls");
    let mut path2 = OwnedPath::new("/");
    let mut path3 = OwnedPath::new("./../../home/");

    path0.canonicalize(&OwnedPath::new("/home/name"));
    path1.canonicalize(&OwnedPath::new("/usr/"));
    path2.canonicalize(&OwnedPath::new("/usr/bin/"));
    path3.canonicalize(&OwnedPath::new("/usr/bin"));

    assert_eq!(path0.as_str(), "/usr/bin/ls");
    assert_eq!(path1.as_str(), "/usr/bin/ls");
    assert_eq!(path2.as_str(), "/");
    assert_eq!(path3.as_str(), "/usr/bin/./../../home/");
}

/// Test Path Splitting
#[test]
pub fn test_path_splitting() {
    use libutils::paths::OwnedPath;

    let path0 = OwnedPath::new("/usr/bin/ls");
    let path1 = OwnedPath::new("bin/ls");
    let path2 = OwnedPath::new("/");
    let path3 = OwnedPath::new("./../../home/");

    assert_eq!(path0.split_last(), (OwnedPath::new("/usr/bin/"), "ls"));
    assert_eq!(path1.split_last(), (OwnedPath::new("bin/"), "ls"));
    assert_eq!(path2.split_last(), (OwnedPath::new(""), ""));
    assert_eq!(path3.split_last(), (OwnedPath::new("./../../"), "home"));
}
