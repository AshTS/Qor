extern crate libutils;

/// Test the Path Iterator
#[test]
pub fn test_path_iterator()
{
    use libutils::paths::OwnedPath;

    let path0 = OwnedPath::new("/usr/bin/ls");
    let path1 = OwnedPath::new("bin/ls");
    let path2 = OwnedPath::new("/");
    let path3 = OwnedPath::new("./../../home/");

    assert_eq!(path0.iter().collect::<Vec<_>>(), vec!["usr", "bin", "ls"]);
    assert_eq!(path1.iter().collect::<Vec<_>>(), vec!["bin", "ls"]);
    assert_eq!(path2.iter().collect::<Vec<_>>(), Vec::<&str>::new());
    assert_eq!(path3.iter().collect::<Vec<_>>(), vec![".", "..", "..", "home"]);

    assert_eq!((&path0).iter().collect::<Vec<_>>(), vec!["usr", "bin", "ls"]);
    assert_eq!((&path1).iter().collect::<Vec<_>>(), vec!["bin", "ls"]);
    assert_eq!((&path2).iter().collect::<Vec<_>>(), Vec::<&str>::new());
    assert_eq!((&path3).iter().collect::<Vec<_>>(), vec![".", "..", "..", "home"]);
}