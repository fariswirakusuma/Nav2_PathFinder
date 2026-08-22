app_name := "BawalPathFinder"
bin_dir := "bin"

[private]
default:
    @just --list

all: build_frontend build_backend

build_backend:
    @echo "=== Membangun Image Docker ROS 2 Backend (Tanpa Cache) ==="
    docker build --no-cache -t nav2_backend:latest -f Dockerfile .

build_frontend:
    @echo "=== Mengompilasi Frontend Rust (Mode Rilis) ==="
    cd Interface && cargo build --release
    mkdir -p {{bin_dir}}
    cp Interface/target/release/BawalPathFinder {{bin_dir}}/{{app_name}}
    @echo "=== Eksekusi frontend siap di {{bin_dir}}/{{app_name}} ==="

run: all
    @echo "=== Meluncurkan Sistem BawalPathFinder ==="
    bash bash/run_all.sh

rebuild_all: stop clean build_backend build_frontend
    @echo "=== Sistem telah di-rebuild total ==="

stop:
    @echo "=== Menghentikan dan Menghapus Kontainer ==="
    -docker rm -f nav2_sim_backend 2>/dev/null
    -pkill -x {{app_name}}

clean: stop map_clean
    @echo "=== Menghapus Binary dan Cache ==="
    rm -rf {{bin_dir}}/{{app_name}}
    cd Interface && cargo clean

map:
    @echo "=== Menyiapkan Peta Simulasi ==="
    pip install Pillow faker
    python3 Test/maps/generate_test_map.py
    @echo "=== Sinkronisasi Peta ke ROS Workspace ==="
    mkdir -p ROS_workspace/src/navigation/maps
    -cp Test/maps/*.yaml ROS_workspace/src/navigation/maps/ 2>/dev/null
    -cp Test/maps/*.png ROS_workspace/src/navigation/maps/ 2>/dev/null

map_clean:
    @echo "=== Menghapus Aset Peta Lama ==="
    rm -f Test/maps/map_*.yaml Test/maps/map_*.png
    rm -f ROS_workspace/src/navigation/maps/map_*.yaml ROS_workspace/src/navigation/maps/map_*.png