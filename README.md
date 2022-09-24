# Segmented file system client in Rust

This is an implementation in Rust of the Segmented File System Client
described in the write-up for
[this lab](https://github.com/UMM-CSci-Systems/Segmented-file-system-client)
for our systems lab course.

The current lab expects students to write this in Java, but I
thought it would be interesting to see what the solution looks like
in Rust. Depending on what happens, we might consider rewriting the
lab so that the students do it in Rust, but I'd definitely want to
work through a solution of my own first, though.

This feels like a good Rust project, especially since we're
dealing with "raw bytes" in the packets.

I also might want to rewrite the server to be in Rust instead of Java,
although I must say that it's nice knowing that I can include the
Jar file for the server in things like the student's starter repo and
know that it will work on pretty much any platform, which wouldn't
be true if the server was written in Rust.
