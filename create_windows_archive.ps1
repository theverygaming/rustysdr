$build_dir=$args[0]

mkdir rustysdr_windows_x64

# main executable
cp $build_dir/release/silly2.exe rustysdr_windows_pkg/

# dynamic libraries
cp $build_dir/release/release/build/libvok-sys-*/out/lib/libvolk.dll rustysdr_windows_pkg/
cp $build_dir/release/build/fftw-src/*/out/libfftw3-3.dll rustysdr_windows_pkg/
cp $build_dir/release/build/fftw-src/*/out/libfftw3f-3.dll rustysdr_windows_pkg/


Compress-Archive -Path rustysdr_windows_pkg/ -DestinationPath rustysdr_windows.zip

rm -Force -Recurse rustysdr_windows_pkg
