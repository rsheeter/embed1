# embed1

## Setup

```shell
# We assume things about directories because we're lazy
$ mkdir ~/oss
$ cd ~/oss
$ git clone git@github.com:google/fonts.git
$ git clone git@github.com:rsheeter/embed1.git
```

## Confirm we're online

```shell
$ cargo run -p read-gf-metadata
...noise...
Read 1911/1911 METADATA.pb files successfully
Read 1682/1682 language files successfully
```

## Make some test images

```shell
$ cargo run -p make_test_images
# Should write lots of /tmp/family.ext.png
$ ls -1 /tmp/*.ttf.png | wc -l
1905
```

~[Lobster render sample](Lobster-Regular.ttf.png)