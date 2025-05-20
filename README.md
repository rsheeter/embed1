# embed1

## Execution

### Setup

```shell
# We assume things about directories because we're lazy
$ mkdir ~/oss
$ cd ~/oss
$ git clone git@github.com:google/fonts.git
$ git clone git@github.com:rsheeter/embed1.git
# Optional, only needed for icons
$ git clone git@github.com:google/material-design-icons.git
```

### Text fonts

```shell
# Confirm we're online
$ cargo run -p read-gf-metadata
...noise...
Read 1911/1911 METADATA.pb files successfully
Read 1682/1682 language files successfully

# Make all the test images
$ cargo build --release -p make_test_images && target/release/make_test_images
$ ls -1 /tmp/test_png/*.png | wc -l
1905

# Process just one family
# jua uses primary_script
$ cargo run -p make_test_images -- --family-filter ofl/jua

# notosanstc uses primary_language
$ cargo run -p make_test_images -- --family-filter ofl/notosanstc
```

![Lobster render sample](Lobster-Regular.ttf.png)
![Shippori Mincho render sample](ShipporiMincho-Regular.ttf.png)

```shell
# Make embedddings
$ cargo build --release -p make_embedding && target/release/make_embedding

# Query embeddings
$ cargo run -p query_embedding -- "korean serif"
```

### Google-style icon font

```shell
# Make all the test images
$ cargo build --release -p make_icon_images && target/release/make_icon_images --icon-font ~/oss/material-design-icons/variablefont/MaterialSymbolsOutlined\[FILL,GRAD,opsz,wght].ttf
$ ls -1 /tmp/icon_png/*.png | wc -l
3649
```

![Material Symbols render sample](namecomment.png)

## References

* https://openai.com/index/clip/
* https://stackoverflow.blog/2023/11/09/an-intuitive-introduction-to-text-embeddings/
* https://github.com/StarlightSearch/EmbedAnything/blob/main/examples/clip.py
