# Dedup

This is a *very* simple CLI tool that I created to be useful for myself and
I'm now sharing with the world. PR's are welcome and maybe I'll expand it if
I feel the need for it. For now:

You create an input list of files using the ```find``` command:

```
find /path -type f > ./input.txt
```

Then you feed that input.txt file to dedup, hope your user can actually read
all the files in the list, and you'll end up with a SQLite3 database that
contains your files with their full path and the SHA256 checksum of each file.

If any file has the same SHA256 checksum you can assume them to be duplicates.

Use at your own peril.
