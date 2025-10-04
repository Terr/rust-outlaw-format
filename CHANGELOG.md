# v2.1

* Add support for TODO items (`[ ]`, `[x]`)
* Fix identing issue with lists placed before the first header
* Output correct line endings on Windows
* Update OS versions used for building releases, also create Apple ARM release

# v2.0

* Rewrote entire formatter, fixes many list indenting issues from v0.1
* Lines that overshoot the max. line length are still split at the next whitespace
* Support for "fenched filetypes" (Outlaw feature for marking blocks for code, with syntax highlighting). Lines within these blocks are not reformatted.
* Benchmarks added


# v1.0

* Initial version. Basic Outlaw file formatting
