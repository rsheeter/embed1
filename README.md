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
# Make all the test images
$ $ cargo build --release -p make_test_images && target/release/make_test_images
$ ls -1 /tmp/test_png/*.png | wc -l
1905

# Process just one family
# jua uses primary_script
$ cargo run -p make_test_images -- --family-filter ofl/jua

# notosanstc uses primary_language
$ cargo run -p make_test_images -- --family-filter ofl/notosanstc
```

![Lobster render sample](Lobster-Regular.ttf.png)

## References

* https://openai.com/index/clip/
* https://stackoverflow.blog/2023/11/09/an-intuitive-introduction-to-text-embeddings/
