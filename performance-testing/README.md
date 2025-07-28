## Some Performance apps for comparison

This folder contains some applications to get performance data of some librairies/crates. The followings can be found here in sub-folders:

- `pugixml` - A C++ xml parser
- `roxmltree` - A Rust crate, read-only xml DOM parser
- `xml5ever` - A Rust crate xml DOM parser
- `xhtml_parser` - A Rust crate, a read-only xml DOM parser

To build and run, do the following:

```
$ cd <sub-folder>
$ ./build.sh
$ ./run.sh
```

The `run.sh` script will run the app 20 times.

The `perdormance-results.xls` contains the test results done by the author, as (to be) presented in the `xhtml_parser` documentation.

Please note that for the xml5ever crate, the resulting data was not published in the documentation as the code used to get some results may not be appropriate.
