// -*- tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- /
/* vim: set shiftwidth=2 tabstop=2 autoindent cindent expandtab: */
/*
   Copyright 2011 notmasteryet

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
*/

// SPDX-License-Identifier: Apache-2.0
// Bounded Rust translation of jpeg-js 0.4.4 decoder.js. See PROVENANCE.md.

use super::{dimensions, Raster};
const MEMORY: usize = 384 * 1024 * 1024;
const MAX_BLOCK_VISITS: usize = 16_000_000;
const Z: [usize; 64] = [
    0, 1, 8, 16, 9, 2, 3, 10, 17, 24, 32, 25, 18, 11, 4, 5, 12, 19, 26, 33, 40, 48, 41, 34, 27, 20,
    13, 6, 7, 14, 21, 28, 35, 42, 49, 56, 57, 50, 43, 36, 29, 22, 15, 23, 30, 37, 44, 51, 58, 59,
    52, 45, 38, 31, 39, 46, 53, 60, 61, 54, 47, 55, 62, 63,
];
type R<T> = Result<T, String>;
fn bad<T>() -> R<T> {
    Err("Unsupported or damaged JPEG".into())
}
// JS bitwise operators apply ToInt32 before the signed shift. All other
// IDCT intermediates fit exactly in JS Number and i64 under our coefficient cap.
fn shr(x: i64, n: u32) -> i64 {
    ((x as i32) >> n) as i64
}
// A port of poppler's IDCT method which in turn is taken from:
// Christoph Loeffler, Adriaan Ligtenberg, George S. Moschytz,
// "Practical Fast 1-D DCT Algorithms with 11 Multiplications",
// IEEE Intl. Conf. on Acoustics, Speech & Signal Processing, 1989, 988-991.
// Retained attribution from the actual Apache-2.0 jpeg-js decoder source.
fn idct(zz: &[i32; 64], qt: &[i32; 64]) -> [u8; 64] {
    let mut p = [0i64; 64];
    for i in 0..64 {
        p[i] = (i64::from(zz[i]) * i64::from(qt[i])) as i32 as i64;
    }
    let (mut v0, mut v1, mut v2, mut v3, mut v4, mut v5, mut v6, mut v7, mut t);
    // inverse DCT on rows
    for i in 0..8 {
        let row = 8 * i;

        // check for all-zero AC coefficients
        if p[1 + row] == 0
            && p[2 + row] == 0
            && p[3 + row] == 0
            && p[4 + row] == 0
            && p[5 + row] == 0
            && p[6 + row] == 0
            && p[7 + row] == 0
        {
            t = shr(5793 * p[row] + 512, 10);
            p[row] = (t) as i32 as i64;
            p[1 + row] = (t) as i32 as i64;
            p[2 + row] = (t) as i32 as i64;
            p[3 + row] = (t) as i32 as i64;
            p[4 + row] = (t) as i32 as i64;
            p[5 + row] = (t) as i32 as i64;
            p[6 + row] = (t) as i32 as i64;
            p[7 + row] = (t) as i32 as i64;
            continue;
        }

        // stage 4
        v0 = shr(5793 * p[row] + 128, 8);
        v1 = shr(5793 * p[4 + row] + 128, 8);
        v2 = p[2 + row];
        v3 = p[6 + row];
        v4 = shr(2896 * (p[1 + row] - p[7 + row]) + 128, 8);
        v7 = shr(2896 * (p[1 + row] + p[7 + row]) + 128, 8);
        v5 = (p[3 + row] as i32).wrapping_shl(4) as i64;
        v6 = (p[5 + row] as i32).wrapping_shl(4) as i64;

        // stage 3
        t = shr(v0 - v1 + 1, 1);
        v0 = shr(v0 + v1 + 1, 1);
        v1 = t;
        t = shr(v2 * 3784 + v3 * 1567 + 128, 8);
        v2 = shr(v2 * 1567 - v3 * 3784 + 128, 8);
        v3 = t;
        t = shr(v4 - v6 + 1, 1);
        v4 = shr(v4 + v6 + 1, 1);
        v6 = t;
        t = shr(v7 + v5 + 1, 1);
        v5 = shr(v7 - v5 + 1, 1);
        v7 = t;

        // stage 2
        t = shr(v0 - v3 + 1, 1);
        v0 = shr(v0 + v3 + 1, 1);
        v3 = t;
        t = shr(v1 - v2 + 1, 1);
        v1 = shr(v1 + v2 + 1, 1);
        v2 = t;
        t = shr(v4 * 2276 + v7 * 3406 + 2048, 12);
        v4 = shr(v4 * 3406 - v7 * 2276 + 2048, 12);
        v7 = t;
        t = shr(v5 * 799 + v6 * 4017 + 2048, 12);
        v5 = shr(v5 * 4017 - v6 * 799 + 2048, 12);
        v6 = t;

        // stage 1
        p[row] = (v0 + v7) as i32 as i64;
        p[7 + row] = (v0 - v7) as i32 as i64;
        p[1 + row] = (v1 + v6) as i32 as i64;
        p[6 + row] = (v1 - v6) as i32 as i64;
        p[2 + row] = (v2 + v5) as i32 as i64;
        p[5 + row] = (v2 - v5) as i32 as i64;
        p[3 + row] = (v3 + v4) as i32 as i64;
        p[4 + row] = (v3 - v4) as i32 as i64;
    }

    // inverse DCT on columns
    for i in 0..8 {
        let col = i;

        // check for all-zero AC coefficients
        if p[8 + col] == 0
            && p[2 * 8 + col] == 0
            && p[3 * 8 + col] == 0
            && p[4 * 8 + col] == 0
            && p[5 * 8 + col] == 0
            && p[6 * 8 + col] == 0
            && p[7 * 8 + col] == 0
        {
            t = shr(5793 * p[i] + 8192, 14);
            p[col] = (t) as i32 as i64;
            p[8 + col] = (t) as i32 as i64;
            p[2 * 8 + col] = (t) as i32 as i64;
            p[3 * 8 + col] = (t) as i32 as i64;
            p[4 * 8 + col] = (t) as i32 as i64;
            p[5 * 8 + col] = (t) as i32 as i64;
            p[6 * 8 + col] = (t) as i32 as i64;
            p[7 * 8 + col] = (t) as i32 as i64;
            continue;
        }

        // stage 4
        v0 = shr(5793 * p[col] + 2048, 12);
        v1 = shr(5793 * p[4 * 8 + col] + 2048, 12);
        v2 = p[2 * 8 + col];
        v3 = p[6 * 8 + col];
        v4 = shr(2896 * (p[8 + col] - p[7 * 8 + col]) + 2048, 12);
        v7 = shr(2896 * (p[8 + col] + p[7 * 8 + col]) + 2048, 12);
        v5 = p[3 * 8 + col];
        v6 = p[5 * 8 + col];

        // stage 3
        t = shr(v0 - v1 + 1, 1);
        v0 = shr(v0 + v1 + 1, 1);
        v1 = t;
        t = shr(v2 * 3784 + v3 * 1567 + 2048, 12);
        v2 = shr(v2 * 1567 - v3 * 3784 + 2048, 12);
        v3 = t;
        t = shr(v4 - v6 + 1, 1);
        v4 = shr(v4 + v6 + 1, 1);
        v6 = t;
        t = shr(v7 + v5 + 1, 1);
        v5 = shr(v7 - v5 + 1, 1);
        v7 = t;

        // stage 2
        t = shr(v0 - v3 + 1, 1);
        v0 = shr(v0 + v3 + 1, 1);
        v3 = t;
        t = shr(v1 - v2 + 1, 1);
        v1 = shr(v1 + v2 + 1, 1);
        v2 = t;
        t = shr(v4 * 2276 + v7 * 3406 + 2048, 12);
        v4 = shr(v4 * 3406 - v7 * 2276 + 2048, 12);
        v7 = t;
        t = shr(v5 * 799 + v6 * 4017 + 2048, 12);
        v5 = shr(v5 * 4017 - v6 * 799 + 2048, 12);
        v6 = t;

        // stage 1
        p[col] = (v0 + v7) as i32 as i64;
        p[7 * 8 + col] = (v0 - v7) as i32 as i64;
        p[8 + col] = (v1 + v6) as i32 as i64;
        p[6 * 8 + col] = (v1 - v6) as i32 as i64;
        p[2 * 8 + col] = (v2 + v5) as i32 as i64;
        p[5 * 8 + col] = (v2 - v5) as i32 as i64;
        p[3 * 8 + col] = (v3 + v4) as i32 as i64;
        p[4 * 8 + col] = (v3 - v4) as i32 as i64;
    }

    let mut out = [0; 64];
    for i in 0..64 {
        out[i] = (128 + shr(p[i] + 8, 4)).clamp(0, 255) as u8;
    }
    out
}
#[derive(Clone)]
struct Huff {
    counts: [u8; 16],
    values: Vec<u8>,
}
impl Huff {
    fn read(&self, b: &mut Bits<'_>) -> R<u8> {
        let (mut code, mut first, mut index) = (0u32, 0u32, 0usize);
        for &count in &self.counts {
            code = (code << 1) | b.bit()? as u32;
            if code >= first && code - first < u32::from(count) {
                return self
                    .values
                    .get(index + (code - first) as usize)
                    .copied()
                    .ok_or_else(|| "Invalid JPEG Huffman code".into());
            }
            index += count as usize;
            first = (first + u32::from(count)) << 1;
        }
        bad()
    }
}
struct Bits<'a> {
    data: &'a [u8],
    pos: usize,
    value: u8,
    left: u8,
}
impl Bits<'_> {
    fn byte(&mut self) -> R<u8> {
        let v = *self.data.get(self.pos).ok_or("Truncated JPEG")?;
        self.pos += 1;
        Ok(v)
    }
    fn bit(&mut self) -> R<i32> {
        if self.left == 0 {
            self.value = self.byte()?;
            if self.value == 255 && self.byte()? != 0 {
                return bad();
            }
            self.left = 8;
        }
        self.left -= 1;
        Ok(i32::from((self.value >> self.left) & 1))
    }
    fn receive(&mut self, n: u8) -> R<i32> {
        if n > 16 {
            return bad();
        }
        let mut v = 0;
        for _ in 0..n {
            v = (v << 1) | self.bit()?;
        }
        Ok(v)
    }
    fn extend(&mut self, n: u8) -> R<i32> {
        if n == 0 {
            return Ok(0);
        }
        let v = self.receive(n)?;
        Ok(if v >= 1 << (n - 1) {
            v
        } else {
            v + (-1 << n) + 1
        })
    }
}
struct Component {
    id: u8,
    h: usize,
    v: usize,
    q: usize,
    bw: usize,
    bh: usize,
    stride: usize,
    blocks: Vec<[i32; 64]>,
    lines: Vec<u8>,
    pred: i32,
    seen: [i8; 64],
}
struct Frame {
    w: usize,
    h: usize,
    mh: usize,
    mv: usize,
    mw: usize,
    mc: usize,
    progressive: bool,
    components: Vec<Component>,
}
struct Scan {
    start: usize,
    end: usize,
    prev: u8,
    low: u8,
    eob: i32,
    state: u8,
    next: i32,
}
impl Scan {
    fn block(
        &mut self,
        b: &mut Bits<'_>,
        zz: &mut [i32; 64],
        pred: &mut i32,
        dc: Option<&Huff>,
        ac: Option<&Huff>,
        progressive: bool,
    ) -> R<()> {
        if !progressive || self.start == 0 {
            if self.prev == 0 {
                let t = dc.ok_or("Missing JPEG DC table")?.read(b)?;
                if t > 11 {
                    return bad();
                }
                let diff = b.extend(t)? * (1 << self.low);
                *pred = pred.checked_add(diff).ok_or("JPEG DC overflow")?;
                if pred.abs() > 1_048_576 {
                    return bad();
                }
                zz[0] = *pred;
            } else {
                zz[0] |= b.bit()? << self.low;
            }
            if progressive {
                return Ok(());
            }
        }
        let ac = ac.ok_or("Missing JPEG AC table")?;
        let mut k = if progressive { self.start } else { 1 };
        if self.prev == 0 {
            if self.eob > 0 {
                self.eob -= 1;
                return Ok(());
            }
            while k <= self.end {
                let rs = ac.read(b)?;
                let n = rs & 15;
                let run = usize::from(rs >> 4);
                if n == 0 {
                    if run < 15 {
                        if progressive {
                            self.eob = b.receive(run as u8)? + (1 << run) - 1;
                        }
                        break;
                    }
                    k += 16;
                    if k > self.end + 1 {
                        return bad();
                    }
                    continue;
                }
                if n > 10 {
                    return bad();
                }
                k += run;
                if k > self.end {
                    return bad();
                }
                zz[Z[k]] = b.extend(n)? * (1 << self.low);
                k += 1;
            }
        } else {
            let mut run = 0;
            while k <= self.end {
                let z = Z[k];
                let direction = if zz[z] < 0 { -1 } else { 1 };
                match self.state {
                    0 => {
                        let rs = ac.read(b)?;
                        let n = rs & 15;
                        run = i32::from(rs >> 4);
                        if n == 0 {
                            if run < 15 {
                                self.eob = b.receive(run as u8)? + (1 << run);
                                self.state = 4;
                            } else {
                                run = 16;
                                self.state = 1;
                            }
                        } else {
                            if n != 1 {
                                return bad();
                            }
                            self.next = b.extend(n)?;
                            self.state = if run != 0 { 2 } else { 3 };
                        }
                        continue;
                    }
                    1 | 2 => {
                        if zz[z] != 0 {
                            zz[z] += (b.bit()? << self.low) * direction;
                        } else {
                            run -= 1;
                            if run == 0 {
                                self.state = if self.state == 2 { 3 } else { 0 };
                            }
                        }
                    }
                    3 => {
                        if zz[z] != 0 {
                            zz[z] += (b.bit()? << self.low) * direction;
                        } else {
                            zz[z] = self.next << self.low;
                            self.state = 0;
                        }
                    }
                    4 => {
                        if zz[z] != 0 {
                            zz[z] += (b.bit()? << self.low) * direction;
                        }
                    }
                    _ => return bad(),
                }
                k += 1;
            }
            if self.state == 4 {
                self.eob -= 1;
                if self.eob == 0 {
                    self.state = 0;
                }
            }
        }
        if zz.iter().any(|v| v.abs() > 1_048_576) {
            return bad();
        }
        Ok(())
    }
}
fn u16be(d: &[u8], p: usize) -> R<usize> {
    let a = d.get(p..p + 2).ok_or("Truncated JPEG")?;
    Ok(usize::from(u16::from_be_bytes([a[0], a[1]])))
}
fn take<'a>(d: &'a [u8], p: &mut usize, n: usize) -> R<&'a [u8]> {
    let a = d
        .get(*p..p.checked_add(n).ok_or("JPEG offset overflow")?)
        .ok_or("Truncated JPEG")?;
    *p += n;
    Ok(a)
}
fn alloc<T: Clone>(n: usize, value: T) -> R<Vec<T>> {
    let mut v = Vec::new();
    v.try_reserve_exact(n)
        .map_err(|_| "JPEG allocation failed")?;
    v.resize(n, value);
    Ok(v)
}
pub(super) fn decode(data: &[u8]) -> R<Raster> {
    let mut pos = 2;
    let mut frame: Option<Frame> = None;
    let mut quant: [Option<[i32; 64]>; 16] = [None; 16];
    let mut tables: [Option<Huff>; 32] = std::array::from_fn(|_| None);
    let mut restart = 0;
    let mut adobe = None;
    let mut visits = 0usize;
    let mut scans = 0;
    loop {
        if take(data, &mut pos, 1)? != [255] {
            return bad();
        }
        let mut marker = take(data, &mut pos, 1)?[0];
        while marker == 255 {
            marker = take(data, &mut pos, 1)?[0];
        }
        if marker == 0xd9 {
            break;
        }
        let length = u16be(data, pos)?;
        pos += 2;
        if length < 2 {
            return bad();
        }
        let segment = take(data, &mut pos, length - 2)?;
        match marker {
            0xe0..=0xef | 0xfe => {
                if marker == 0xee && segment.starts_with(b"Adobe\0") {
                    adobe = Some(*segment.get(11).ok_or("Truncated Adobe JPEG marker")?);
                }
            }
            0xdb => {
                let mut p = 0;
                while p < segment.len() {
                    let spec = take(segment, &mut p, 1)?[0];
                    if spec >> 4 > 1 {
                        return bad();
                    }
                    let mut q = [0; 64];
                    for &z in &Z {
                        q[z] = if spec >> 4 == 0 {
                            i32::from(take(segment, &mut p, 1)?[0])
                        } else {
                            let v = u16be(segment, p)?;
                            p += 2;
                            v as i32
                        };
                        if q[z] == 0 {
                            return bad();
                        }
                    }
                    quant[usize::from(spec & 15)] = Some(q);
                }
            }
            0xc0..=0xc2 => {
                if frame.is_some() || segment.len() < 6 || segment[0] != 8 {
                    return bad();
                }
                let h = u16be(segment, 1)?;
                let w = u16be(segment, 3)?;
                dimensions(w as u32, h as u32)?;
                let count = usize::from(segment[5]);
                if ![1, 3, 4].contains(&count) || segment.len() != 6 + count * 3 {
                    return bad();
                }
                let mut cs = vec![];
                let (mut mh, mut mv) = (1, 1);
                for c in segment[6..].chunks_exact(3) {
                    let h = usize::from(c[1] >> 4);
                    let v = usize::from(c[1] & 15);
                    if !(1..=4).contains(&h)
                        || !(1..=4).contains(&v)
                        || c[2] > 15
                        || cs.iter().any(|a: &Component| a.id == c[0])
                    {
                        return bad();
                    }
                    mh = mh.max(h);
                    mv = mv.max(v);
                    cs.push(Component {
                        id: c[0],
                        h,
                        v,
                        q: c[2] as usize,
                        bw: 0,
                        bh: 0,
                        stride: 0,
                        blocks: vec![],
                        lines: vec![],
                        pred: 0,
                        seen: [-1; 64],
                    });
                }
                let mw = w.div_ceil(8 * mh);
                let mc = h.div_ceil(8 * mv);
                let mut memory = w * h * 4;
                for c in &mut cs {
                    c.bw = (w.div_ceil(8) * c.h).div_ceil(mh);
                    c.bh = (h.div_ceil(8) * c.v).div_ceil(mv);
                    c.stride = mw * c.h;
                    let blocks = c.stride * mc * c.v;
                    memory = memory
                        .checked_add(blocks * 256 + c.bw * c.bh * 64)
                        .ok_or("JPEG allocation overflow")?;
                    if memory > MEMORY {
                        return Err("JPEG working buffers exceed 384 MiB".into());
                    }
                }
                for c in &mut cs {
                    c.blocks = alloc(c.stride * mc * c.v, [0; 64])?;
                }
                frame = Some(Frame {
                    w,
                    h,
                    mh,
                    mv,
                    mw,
                    mc,
                    progressive: marker == 0xc2,
                    components: cs,
                });
            }
            0xc4 => {
                let mut p = 0;
                while p < segment.len() {
                    let spec = take(segment, &mut p, 1)?[0];
                    if spec >> 4 > 1 {
                        return bad();
                    }
                    let counts: [u8; 16] = take(segment, &mut p, 16)?
                        .try_into()
                        .map_err(|_| "Invalid JPEG table")?;
                    let mut slots = 1i32;
                    let mut sum = 0;
                    for &n in &counts {
                        slots = slots * 2 - i32::from(n);
                        if slots < 0 {
                            return bad();
                        }
                        sum += n as usize;
                    }
                    if sum == 0 || sum > 256 {
                        return bad();
                    }
                    let values = take(segment, &mut p, sum)?.to_vec();
                    tables[(spec >> 4) as usize * 16 + (spec & 15) as usize] =
                        Some(Huff { counts, values });
                }
            }
            0xdc => {
                // jpeg-js ignores DNL; the already bounded SOF dimensions govern.
                if segment.len() != 2 {
                    return bad();
                }
            }
            0xdd => {
                if segment.len() != 2 {
                    return bad();
                }
                restart = u16be(segment, 0)?;
            }
            0xda => {
                scans += 1;
                if scans > 256 {
                    return Err("JPEG exceeds 256 scans".into());
                }
                let f = frame.as_mut().ok_or("JPEG scan before frame")?;
                let count = usize::from(*segment.first().ok_or("Empty JPEG scan")?);
                if count == 0 || count > f.components.len() || segment.len() != 1 + count * 2 + 3 {
                    return bad();
                }
                let mut selectors = vec![];
                for s in segment[1..1 + count * 2].chunks_exact(2) {
                    let i = f
                        .components
                        .iter()
                        .position(|c| c.id == s[0])
                        .ok_or("Unknown JPEG scan component")?;
                    if selectors.iter().any(|&(j, _, _)| j == i) {
                        return bad();
                    }
                    selectors.push((i, usize::from(s[1] >> 4), 16 + usize::from(s[1] & 15)));
                }
                let tail = &segment[1 + count * 2..];
                let start = tail[0] as usize;
                let end = tail[1] as usize;
                let prev = tail[2] >> 4;
                let low = tail[2] & 15;
                if start > end
                    || end > 63
                    || low > 13
                    || prev > 13
                    || (prev != 0 && prev != low + 1)
                    || (f.progressive && ((start == 0 && end != 0) || (start > 0 && count != 1)))
                    || (!f.progressive && (start != 0 || end != 63 || prev != 0 || low != 0))
                {
                    return bad();
                }
                for &(i, _, _) in &selectors {
                    for s in start..=end {
                        let seen = &mut f.components[i].seen[s];
                        if (prev == 0 && *seen != -1) || (prev != 0 && *seen != prev as i8) {
                            return bad();
                        }
                        *seen = low as i8;
                    }
                }
                let total = if count == 1 {
                    let c = &f.components[selectors[0].0];
                    c.bw * c.bh
                } else {
                    f.mw * f.mc
                };
                let mut bits = Bits {
                    data,
                    pos,
                    value: 0,
                    left: 0,
                };
                let mut mcu = 0;
                let mut rst = 0;
                while mcu < total {
                    for &(i, _, _) in &selectors {
                        f.components[i].pred = 0;
                    }
                    let mut scan = Scan {
                        start,
                        end,
                        prev,
                        low,
                        eob: 0,
                        state: 0,
                        next: 0,
                    };
                    let limit = if restart == 0 {
                        total
                    } else {
                        (mcu + restart).min(total)
                    };
                    while mcu < limit {
                        for &(i, dc, ac) in &selectors {
                            let c = &mut f.components[i];
                            let (rows, cols) = if count == 1 { (1, 1) } else { (c.v, c.h) };
                            for row in 0..rows {
                                for col in 0..cols {
                                    visits += 1;
                                    if visits > MAX_BLOCK_VISITS {
                                        return Err("JPEG exceeds 16,000,000 block visits".into());
                                    }
                                    let k = if count == 1 {
                                        (mcu / c.bw) * c.stride + mcu % c.bw
                                    } else {
                                        ((mcu / f.mw) * c.v + row) * c.stride
                                            + (mcu % f.mw) * c.h
                                            + col
                                    };
                                    let zz =
                                        c.blocks.get_mut(k).ok_or("JPEG block outside frame")?;
                                    scan.block(
                                        &mut bits,
                                        zz,
                                        &mut c.pred,
                                        tables[dc].as_ref(),
                                        tables[ac].as_ref(),
                                        f.progressive,
                                    )?;
                                }
                            }
                        }
                        mcu += 1;
                    }
                    if scan.eob != 0 || scan.state != 0 {
                        return bad();
                    }
                    bits.left = 0;
                    if mcu < total {
                        if take(data, &mut bits.pos, 2)? != [255, 0xd0 + rst] {
                            return bad();
                        }
                        rst = (rst + 1) % 8;
                    }
                }
                pos = bits.pos;
            }
            _ => return bad(),
        }
    }
    let mut f = frame.ok_or("Missing JPEG frame")?;
    if scans == 0 || f.components.iter().any(|c| c.seen[0] < 0) {
        return bad();
    }
    for c in &mut f.components {
        let q = quant[c.q]
            .as_ref()
            .ok_or("Missing JPEG quantization table")?;
        c.lines = alloc(c.bw * c.bh * 64, 0)?;
        for by in 0..c.bh {
            for bx in 0..c.bw {
                let block = idct(&c.blocks[by * c.stride + bx], q);
                for y in 0..8 {
                    let k = (by * 8 + y) * c.bw * 8 + bx * 8;
                    c.lines[k..k + 8].copy_from_slice(&block[y * 8..y * 8 + 8]);
                }
            }
        }
        c.blocks = Vec::new();
    }
    let mut rgba = alloc(f.w * f.h * 4, 0)?;
    for y in 0..f.h {
        for x in 0..f.w {
            let mut v = [0f64; 4];
            for (i, c) in f.components.iter().enumerate() {
                let xx = (x as f64 * (c.h as f64 / f.mh as f64)) as usize;
                let yy = (y as f64 * (c.v as f64 / f.mv as f64)) as usize;
                v[i] = f64::from(
                    *c.lines
                        .get(yy * c.bw * 8 + xx)
                        .ok_or("JPEG sample outside frame")?,
                );
            }
            let rgb = match f.components.len() {
                1 => [v[0] as u8; 3],
                3 => {
                    let a = ycbcr(v);
                    [a[0] as u8, a[1] as u8, a[2] as u8]
                }
                4 => {
                    let transform =
                        adobe.ok_or("Unsupported four-component JPEG without Adobe marker")? != 0;
                    let cmy = if transform {
                        let rgb = ycbcr(v);
                        // Preserve both JS subtractions before the Uint8Array store;
                        // algebraic cancellation can change floating truncation boundaries.
                        rgb.map(|a| ((255. - (255. - a)) as u8) as f64)
                    } else {
                        [255. - v[0], 255. - v[1], 255. - v[2]]
                    };
                    let k = 255. - v[3];
                    cmy.map(|c| (255. - (c * (1. - k / 255.) + k).clamp(0., 255.)) as u8)
                }
                _ => return bad(),
            };
            let k = (y * f.w + x) * 4;
            rgba[k..k + 4].copy_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
        }
    }
    Ok(Raster {
        width: f.w as u32,
        height: f.h as u32,
        rgba,
    })
}
fn ycbcr(v: [f64; 4]) -> [f64; 3] {
    let [y, cb, cr, _] = v;
    [
        (y + 1.402 * (cr - 128.)).clamp(0., 255.),
        (y - 0.3441363 * (cb - 128.) - 0.71413636 * (cr - 128.)).clamp(0., 255.),
        (y + 1.772 * (cb - 128.)).clamp(0., 255.),
    ]
}
