# ELF loader

> 细节请看 RuxOS 手册.

## 如何运行

1. 使用 Musl 编译 `rootfs/` 下的文件.

```sh
cd rootfs/
musl-gcc libadd.c -shared -o lib/libadd.so
musl-gcc hello.c -Llib -ladd -o bin/hello
```

2. 将 Musl 动态链接器放入 `rootfs/lib` 下.

3. 运行

使用 `ruxgo` 运行

```sh
# 在 apps/c/dl 目录下
ruxgo -b && ruxgo -r
```

使用 `make` 运行

```sh
# 在 RuxOS 目录下.
make run ARCH=aarch64 A=apps/c/dl V9P=y MUSL=y LOG=debug
```
