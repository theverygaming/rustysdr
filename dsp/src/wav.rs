use std::io;
use std::io::SeekFrom;
use volk_rs::{vec::AlignedVec, Complex};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum WavSampleFormat {
    U8,
    S16,
    S24,
    S32,
    F32,
    F64,
}

fn get_sample_format_bytes(format: WavSampleFormat) -> usize {
    match format {
        WavSampleFormat::U8 => {
            return 1;
        }
        WavSampleFormat::S16 => {
            return 2;
        }
        WavSampleFormat::S24 => {
            return 3;
        }
        WavSampleFormat::S32 => {
            return 4;
        }
        WavSampleFormat::F32 => {
            return 4;
        }
        WavSampleFormat::F64 => {
            return 8;
        }
    }
}

struct WavInfo {
    samplerate: u32,
    channels: u16,
    data_start: u64,
    //bytes_per_sample: u16,
    format: WavSampleFormat,
}

pub struct Reader<R> {
    reader: R,
    buffer: AlignedVec<u8>,
    info: WavInfo,
    infinite: bool,
}

impl<R: io::Read + io::Seek> Reader<R> {
    fn read_header(reader: &mut R) -> Result<WavInfo, io::Error> {
        let err = io::Error::new(io::ErrorKind::Other, "WAV: invalid/unsupported WAV header");

        let mut v4 = [0u8; 4];
        let mut v4_2 = [0u8; 4];
        let mut v2 = [0u8; 2];

        reader.seek(SeekFrom::Start(0))?;
        reader.read_exact(&mut v4)?;
        if &v4 != b"RIFF" {
            return Err(err);
        }
        reader.seek(SeekFrom::Start(8))?;
        reader.read_exact(&mut v4)?;
        if &v4 != b"WAVE" {
            return Err(err);
        }

        // find and read "fmt " chunk
        let mut fmtlen: u32;
        loop {
            reader.read_exact(&mut v4)?;
            reader.read_exact(&mut v4_2)?;
            fmtlen = u32::from_le_bytes(v4_2);
            if &v4 == b"fmt " {
                break;
            }
            reader.seek(SeekFrom::Current(fmtlen as i64))?;
        }
        reader.read_exact(&mut v2)?;
        let format = u16::from_le_bytes(v2);

        if !(format == 1 || format == 3) || fmtlen < 16 {
            return Err(err);
        }

        reader.read_exact(&mut v2)?;
        let channels = u16::from_le_bytes(v2);
        reader.read_exact(&mut v4)?;
        let samplerate = u32::from_le_bytes(v4);
        reader.seek(SeekFrom::Current(6))?; // skip block align and byte rate fields
        reader.read_exact(&mut v2)?;
        let bitspersample = u16::from_le_bytes(v2);

        // find "data" chunk
        reader.seek(SeekFrom::Start(12))?;
        loop {
            reader.read_exact(&mut v4)?;
            reader.read_exact(&mut v4_2)?;
            let len = u32::from_le_bytes(v4_2);
            if &v4 == b"data" {
                break;
            }
            reader.seek(SeekFrom::Current(len as i64))?;
        }

        let data_start = reader.seek(SeekFrom::Current(0))?;

        let sfmt: WavSampleFormat = match format {
            // PCM
            1 => match bitspersample {
                8 => WavSampleFormat::U8,
                16 => WavSampleFormat::S16,
                24 => WavSampleFormat::S24,
                32 => WavSampleFormat::S32,
                _ => return Err(err),
            },
            // IEEE float
            3 => match bitspersample {
                32 => WavSampleFormat::F32,
                64 => WavSampleFormat::F64,
                _ => return Err(err),
            },
            _ => return Err(err),
        };

        return Ok(WavInfo {
            samplerate: samplerate,
            channels: channels,
            data_start: data_start,
            //bytes_per_sample: bitspersample / 8,
            format: sfmt,
        });
    }

    fn read_buffer(&mut self, bytes: usize) -> Result<usize, io::Error> {
        assert!(self.buffer.len() >= bytes, "WAV: read_buffer invalid length");
        let must_read = bytes;
        let mut leftover = bytes;
        let mut has_to_work = true;
        loop {
            let read = match self.reader.read(&mut self.buffer[must_read - leftover..must_read]) {
                Ok(l) => l,
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::Interrupted {
                        continue;
                    }
                    return Err(e);
                }
            };
            leftover -= read;
            if leftover == 0 {
                break;
            }

            if read == 0 {
                if self.infinite {
                    if has_to_work {
                        return Err(io::Error::new(io::ErrorKind::Other, "WAV: no data"));
                    }
                    self.reader.seek(SeekFrom::Start(self.info.data_start))?;
                    has_to_work = true;
                } else {
                    if has_to_work {
                        return Err(io::Error::new(io::ErrorKind::Other, "WAV: end of file"));
                    }
                    return Ok(must_read - leftover);
                }
            } else {
                has_to_work = false;
            }
        }
        Ok(must_read)
    }

    pub fn new(mut reader: R, infinite: bool) -> Result<Self, io::Error> {
        let info = Self::read_header(&mut reader)?;
        let buf_size = 100000;
        let misalign = buf_size % get_sample_format_bytes(info.format);
        let buf = AlignedVec::from_elem(0, (buf_size - misalign) * info.channels as usize);

        Ok(Reader {
            reader: reader,
            buffer: buf,
            info: info,
            infinite: infinite,
        })
    }

    pub fn get_channels(&self) -> u16 {
        self.info.channels
    }

    pub fn get_samplerate(&self) -> u32 {
        self.info.samplerate
    }

    pub fn get_sample_format(&self) -> WavSampleFormat {
        self.info.format
    }

    pub fn get_sample_count(&mut self) -> Result<u64, io::Error> {
        let pos = self.reader.seek(SeekFrom::Current(0))?;
        let fsize = self.reader.seek(SeekFrom::End(0))? - self.info.data_start;
        self.reader.seek(SeekFrom::Start(pos))?;
        Ok((fsize / get_sample_format_bytes(self.info.format) as u64) / self.info.channels as u64)
    }

    pub fn get_samples_read(&mut self) -> Result<u64, io::Error> {
        let pos = self.reader.seek(SeekFrom::Current(0))? - self.info.data_start;
        Ok((pos / get_sample_format_bytes(self.info.format) as u64) / self.info.channels as u64)
    }

    pub fn read_samples(&mut self, arr: &mut [f32]) -> Result<(), io::Error> {
        assert!(arr.len() % (self.info.channels as usize) == 0, "WAV: array length must be divisible by channel count");
        let mut leftover = arr.len();
        let must_read = arr.len();

        loop {
            let mut leftover_bytes = std::cmp::min(self.buffer.len(), leftover * get_sample_format_bytes(self.info.format));
            leftover_bytes = self.read_buffer(leftover_bytes)?;
            if leftover_bytes % get_sample_format_bytes(self.info.format) != 0 {
                return Err(io::Error::new(io::ErrorKind::Other, "WAV: invalid file (unexpected EOF)"));
            }
            let read_samples = leftover_bytes / get_sample_format_bytes(self.info.format);

            match self.info.format {
                WavSampleFormat::U8 => {
                    let scalar = 1.0 / 127.5;
                    let mut n: usize = 0;
                    for e in &mut arr[must_read - leftover..(must_read - leftover) + read_samples].iter_mut() {
                        *e = (self.buffer[0..leftover_bytes][n] as f32 * scalar) - 1.0;
                        n += 1;
                    }
                }
                WavSampleFormat::S16 => {
                    if cfg!(target_endian = "big") {
                        volk_rs::kernels::volk_16u_byteswap_u8(&mut self.buffer[0..leftover_bytes]);
                    }
                    volk_rs::kernels::volk_16i_s32f_convert_32f_u8(&self.buffer[0..leftover_bytes], &mut arr[must_read - leftover..(must_read - leftover) + read_samples], 32768.0);
                }
                WavSampleFormat::S24 => {
                    let scalar = 1.0 / 8388608.0;
                    for i in 0..(leftover_bytes / 3) {
                        let mut n: i32 = self.buffer[i * 3] as i32 | ((self.buffer[(i * 3) + 1] as i32) << 8) | ((self.buffer[(i * 3) + 2] as i32) << 16);
                        // sign-extend
                        n <<= 8;
                        n >>= 8;
                        arr[must_read - leftover..(must_read - leftover) + read_samples][i] = (n as f32) * scalar;
                    }
                }
                WavSampleFormat::S32 => {
                    if cfg!(target_endian = "big") {
                        volk_rs::kernels::volk_32u_byteswap_u8(&mut self.buffer[0..leftover_bytes]);
                    }
                    volk_rs::kernels::volk_32i_s32f_convert_32f_u8(&self.buffer[0..leftover_bytes], &mut arr[must_read - leftover..(must_read - leftover) + read_samples], 2147483648.0);
                }
                WavSampleFormat::F32 => {
                    if cfg!(target_endian = "big") {
                        volk_rs::kernels::volk_32u_byteswap_u8(&mut self.buffer[0..leftover_bytes]);
                    }
                    let mut n: usize = 0;
                    for e in &mut arr[must_read - leftover..(must_read - leftover) + read_samples].iter_mut() {
                        *e = f32::from_ne_bytes(self.buffer[n..n + 4].try_into().unwrap());
                        n += 4;
                    }
                }
                WavSampleFormat::F64 => {
                    if cfg!(target_endian = "big") {
                        volk_rs::kernels::volk_64u_byteswap_u8(&mut self.buffer[0..leftover_bytes]);
                    }
                    let mut n: usize = 0;
                    for e in &mut arr[must_read - leftover..(must_read - leftover) + read_samples].iter_mut() {
                        *e = f64::from_ne_bytes(self.buffer[n..n + 8].try_into().unwrap()) as f32;
                        n += 8;
                    }
                }
            }

            leftover -= read_samples;

            if leftover == 0 {
                break;
            }
        }
        Ok(())
    }

    pub fn read_complex(&mut self, arr: &mut [Complex<f32>]) -> Result<(), io::Error> {
        assert!(self.info.channels == 2, "WAV: need exactly 2 channels in wav to read complex");
        unsafe {
            self.read_samples(std::slice::from_raw_parts_mut(arr.as_mut_ptr() as *mut f32, arr.len() * 2))?;
        }
        Ok(())
    }
}

pub struct Writer<W> {
    writer: W,
    buffer: AlignedVec<u8>,
    format: WavSampleFormat,
    channels: u16,
    samples_written: u64,
}

impl<W: io::Write + io::Seek> Writer<W> {
    fn write_header(writer: &mut W, samplerate: u32, channels: u16, format: WavSampleFormat) -> Result<(), io::Error> {
        writer.write(b"RIFF")?;
        writer.write(&(36 as u32).to_le_bytes())?;
        writer.write(b"WAVE")?;

        writer.write(b"fmt ")?;
        // length
        writer.write(&(16 as u32).to_le_bytes())?;
        // sample format
        writer.write(
            &(match format {
                WavSampleFormat::F32 => 3,
                WavSampleFormat::F64 => 3,
                _ => 1,
            } as u16)
                .to_le_bytes(),
        )?;
        // channels
        writer.write(&(channels as u16).to_le_bytes())?;
        // samplerate
        writer.write(&(samplerate as u32).to_le_bytes())?;
        // bytes per second
        writer.write(&(samplerate * get_sample_format_bytes(format) as u32 * channels as u32).to_le_bytes())?;
        // block align
        writer.write(&(get_sample_format_bytes(format) as u16 * channels).to_le_bytes())?;
        // bits per sample
        writer.write(&(get_sample_format_bytes(format) as u16 * 8).to_le_bytes())?;

        writer.write(b"data")?;
        // length
        writer.write(&(0 as u32).to_le_bytes())?;

        Ok(())
    }

    // FXIME: flush on destroy
    pub fn flush(&mut self) -> Result<(), io::Error> {
        let current = self.writer.seek(SeekFrom::Current(0))?;
        self.writer.flush()?;

        let sampsize: u32 = (self.samples_written * get_sample_format_bytes(self.format) as u64).try_into().unwrap_or(u32::MAX);

        let fsize = match sampsize.checked_add(36) {
            Some(x) => x,
            None => u32::MAX,
        };

        self.writer.seek(SeekFrom::Start(4))?;
        self.writer.write(&fsize.to_le_bytes())?;

        self.writer.seek(SeekFrom::Start(40))?;
        self.writer.write(&sampsize.to_le_bytes())?;

        self.writer.seek(SeekFrom::Start(current))?;
        self.writer.flush()?;
        Ok(())
    }

    pub fn new(mut writer: W, samplerate: u32, channels: u16, format: WavSampleFormat) -> Result<Self, io::Error> {
        Self::write_header(&mut writer, samplerate, channels, format)?;
        let buf = AlignedVec::from_elem(0, 100000 * channels as usize); // buffer size must be divisible by 8, as that is the largest possible sample size

        Ok(Writer {
            writer: writer,
            buffer: buf,
            format: format,
            channels: channels,
            samples_written: 0,
        })
    }

    pub fn write_samples(&mut self, arr: &[f32]) -> Result<(), io::Error> {
        assert!(arr.len() % (self.channels as usize) == 0, "WAV: array length must be divisible by channel count");
        let mut leftover = arr.len();
        let must_write = arr.len();
        loop {
            let leftover_bytes = std::cmp::min(self.buffer.len(), leftover * get_sample_format_bytes(self.format));
            let write_samples = leftover_bytes / get_sample_format_bytes(self.format);

            match self.format {
                WavSampleFormat::U8 => {
                    let mut n: usize = 0;
                    for e in arr[must_write - leftover..(must_write - leftover) + write_samples].iter() {
                        self.buffer[0..leftover_bytes][n] = (((e + 1.0) * 0.5) * 255.0) as u8;
                        n += 1;
                    }
                }
                WavSampleFormat::S16 => {
                    volk_rs::kernels::volk_32f_s32f_convert_16i_u8(&arr[must_write - leftover..(must_write - leftover) + write_samples], &mut self.buffer[0..leftover_bytes], 32767.0);
                    if cfg!(target_endian = "big") {
                        volk_rs::kernels::volk_16u_byteswap_u8(&mut self.buffer[0..leftover_bytes]);
                    }
                }
                WavSampleFormat::S24 => return Err(io::Error::new(io::ErrorKind::Other, "WAV: cannot write S24")),
                WavSampleFormat::S32 => {
                    volk_rs::kernels::volk_32f_s32f_convert_32i_u8(&arr[must_write - leftover..(must_write - leftover) + write_samples], &mut self.buffer[0..leftover_bytes], 2147483647.0);
                    if cfg!(target_endian = "big") {
                        volk_rs::kernels::volk_16u_byteswap_u8(&mut self.buffer[0..leftover_bytes]);
                    }
                }
                WavSampleFormat::F32 => {
                    let mut n: usize = 0;
                    for e in arr[must_write - leftover..(must_write - leftover) + write_samples].iter() {
                        self.buffer[0..leftover_bytes][n..n + 4].copy_from_slice(&e.to_le_bytes());
                        n += 4;
                    }
                }
                WavSampleFormat::F64 => {
                    let mut n: usize = 0;
                    for e in arr[must_write - leftover..(must_write - leftover) + write_samples].iter() {
                        self.buffer[0..leftover_bytes][n..n + 8].copy_from_slice(&(*e as f64).to_le_bytes());
                        n += 8;
                    }
                }
            }

            self.writer.write(&self.buffer[0..leftover_bytes])?;
            self.samples_written += write_samples as u64;

            leftover -= write_samples;

            if leftover == 0 {
                break;
            }
        }

        Ok(())
    }

    pub fn write_complex(&mut self, arr: &[Complex<f32>]) -> Result<(), io::Error> {
        assert!(self.channels == 2, "WAV: need exactly 2 channels in wav to write complex");
        unsafe {
            self.write_samples(std::slice::from_raw_parts(arr.as_ptr() as *const f32, arr.len() * 2))?;
        }
        Ok(())
    }
}
