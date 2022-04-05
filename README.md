[![Build & Test](https://github.com/AndreasOM/cheval/actions/workflows/build_and_test.yml/badge.svg?branch=main)](https://github.com/AndreasOM/cheval/actions/workflows/build_and_test.yml)

# cheval
Info renderer in rust targeting
- png
- framebuffer
- window


## Examples

![Example stream overlay](docs/window.png)

![Animated stripes with alpha](docs/alpha_stripes.apng)


Hint:
Animation created with ...
```
mkdir anim
for f in window_*.png; do echo ${f}; gm convert ${f} -resize 50% anim/${f}; done
gm convert -size 960x540 xc:Black PNG32:black.png
cat black.png anim/window_*.png| ffmpeg -framerate 30 -f image2pipe -i - -plays 0 window.apng -y
```

More example configurations can be found on [github](https://github.com/AndreasOM/cheval-example-configs).
