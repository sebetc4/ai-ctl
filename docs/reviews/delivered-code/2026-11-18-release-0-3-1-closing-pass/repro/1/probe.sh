#!/bin/sh
# Finding 1: a server ai-ctl built loads a library from the directory doctor
# runs it in, through the empty element CMake leaves at the end of its RUNPATH.
# Usage: probe.sh [ai-ctl binary], `ai-ctl` on the PATH by default.
# Runs only `ai-ctl doctor`, gcc and cmake, in a folder of its own; the
# planted library is harmless: it records which program loaded it, then ends
# that program before its main.
set -u
export LC_ALL=C
AI_CTL=${1:-ai-ctl}
WORK=$(mktemp -d)
CUDA_LIB=$(readlink -f /usr/local/cuda)/targets/x86_64-linux/lib
mkdir "$WORK/plant" "$WORK/elsewhere" "$WORK/src"

cat > "$WORK/src/planted.c" <<'EOF'
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
__attribute__((constructor)) static void planted(void) {
    char exe[512] = {0}, cwd[512] = {0}, line[1200];
    readlink("/proc/self/exe", exe, sizeof exe - 1);
    if (!getcwd(cwd, sizeof cwd)) cwd[0] = 0;
    const char *out = getenv("PLANTED_MARKERS");
    int fd = open(out ? out : "/dev/null", O_WRONLY | O_CREAT | O_APPEND, 0600);
    if (fd >= 0) {
        int n = snprintf(line, sizeof line, "planted code ran as uid %d in %s, cwd %s\n",
                         (int)getuid(), exe, cwd);
        if (write(fd, line, n) < 0) {}
        close(fd);
    }
    _exit(0);
}
EOF
gcc -shared -fPIC -Wl,-soname,libgomp.so.1 -o "$WORK/plant/libgomp.so.1" "$WORK/src/planted.c"
export PLANTED_MARKERS="$WORK/markers"

echo "== 1. The installed builds' RUNPATH"
for binary in ~/.local/share/ai-ctl/builds/*/current/*; do
    case $(file -b "$binary") in ELF*) ;; *) continue ;; esac
    printf '%s: %s\n' "$binary" "$(readelf -d "$binary" | grep -o 'RUNPATH.*')"
done

echo "== 2. glibc reads an empty RUNPATH element as the working directory"
printf 'int main(void) { return 0; }\n' > "$WORK/src/victim.c"
gcc -fopenmp -o "$WORK/victim" "$WORK/src/victim.c" -Wl,--enable-new-dtags,-rpath,"$CUDA_LIB":
(cd "$WORK/elsewhere" && "$WORK/victim"; echo "from another directory: exit $?")
(cd "$WORK/plant" && "$WORK/victim" 2>/dev/null; echo "from the planted directory: exit $?")

echo "== 3. ai-ctl doctor run from the planted directory"
(cd "$WORK/plant" && "$AI_CTL" doctor 2>&1 \
    | grep -E '^ +[a-z]+ +(llama|whisper|stable-diffusion|audio)\.cpp ' | cut -c1-160)

echo "== markers: each line is planted code that ran"
cat "$WORK/markers" 2>/dev/null || echo "none"

echo "== 4. CMake: which options end the RUNPATH without an empty element"
mkdir "$WORK/cm"
cat > "$WORK/cm/CMakeLists.txt" <<EOF
cmake_minimum_required(VERSION 3.14)
project(victim C)
add_executable(victim $WORK/src/victim.c)
target_link_libraries(victim $CUDA_LIB/libcudart.so)
install(TARGETS victim RUNTIME)
EOF
for generator in Ninja "Unix Makefiles"; do
    for options in "" "-DCMAKE_NO_BUILTIN_CHRPATH=ON" \
        "-DCMAKE_BUILD_WITH_INSTALL_RPATH=ON -DCMAKE_INSTALL_RPATH_USE_LINK_PATH=ON"; do
        tree=$(mktemp -d -p "$WORK")
        # shellcheck disable=SC2086 # the options are words
        if cmake -S "$WORK/cm" -B "$tree" -G "$generator" $options >"$tree.log" 2>&1 \
            && cmake --build "$tree" >>"$tree.log" 2>&1; then
            result=$(readelf -d "$tree/victim" | grep -o 'RUNPATH.*')
        else
            result="configure or build failed: $(grep -m1 -A1 'CMake Error' "$tree.log" | tail -1)"
        fi
        printf '%s, %s: %s\n' "$generator" "${options:-default}" "$result"
    done
done
echo "working files: $WORK"
