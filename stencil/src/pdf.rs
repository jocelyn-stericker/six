//! This is a heavily modified version of the pdfpdf crate by Benjamin Kimock <kimockb@gmail.com> (aka. saethlin)
//! This is lightly adapted from https://github.com/servo/pathfinder/tree/master/export/src

use crate::components::{CombineStencil, Stencil, StencilMap};
use deflate::Compression;
use kurbo::{Affine, BezPath, PathEl, Point, Size};
use specs::Entity;
use std::collections::HashMap;
use std::io::{self, Write};

struct Counter<T> {
    inner: T,
    count: u64,
}
impl<T> Counter<T> {
    pub fn new(inner: T) -> Counter<T> {
        Counter { inner, count: 0 }
    }
    pub fn pos(&self) -> u64 {
        self.count
    }
}
impl<W: Write> Write for Counter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self.inner.write(buf) {
            Ok(n) => {
                self.count += n as u64;
                Ok(n)
            }
            Err(e) => Err(e),
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.inner.write_all(buf)?;
        self.count += buf.len() as u64;
        Ok(())
    }
}

/// Represents a PDF internal object
struct PdfObject {
    contents: Vec<u8>,
    is_page: bool,
    is_xobject: bool,
    offset: Option<u64>,
}

/// The top-level struct that represents a (partially) in-memory PDF file
pub struct Pdf {
    page_buffer: Vec<u8>,
    objects: Vec<PdfObject>,
    page_size: Option<Size>,
    compression: Option<Compression>,
    fill_color: Vec<(f32, f32, f32)>,
    file: Option<usize>,
}

impl Default for Pdf {
    fn default() -> Self {
        Self::new()
    }
}

impl Pdf {
    /// Create a new blank PDF document
    #[inline]
    pub fn new() -> Self {
        Self {
            page_buffer: Vec::new(),
            objects: vec![
                PdfObject {
                    contents: Vec::new(),
                    is_page: false,
                    is_xobject: false,
                    offset: None,
                },
                PdfObject {
                    contents: Vec::new(),
                    is_page: false,
                    is_xobject: false,
                    offset: None,
                },
            ],
            page_size: None,
            compression: Some(Compression::Fast),
            fill_color: Vec::new(),
            file: None,
        }
    }

    fn write_fill_color(&mut self) {
        let color = self.fill_color.last().unwrap_or(&(0.0, 0.0, 0.0));
        writeln!(self.page_buffer, "{} {} {} rg", color.0, color.1, color.2).unwrap();
    }

    /// Set the color for all subsequent drawing operations
    #[inline]
    pub fn push_fill_color(&mut self, r: f32, g: f32, b: f32) {
        self.fill_color.push((r, g, b));
        self.write_fill_color();
    }

    pub fn pop_fill_color(&mut self) {
        if !self.fill_color.is_empty() {
            self.fill_color.pop();
        }
        self.write_fill_color();
    }

    pub fn write_text(&mut self, font_size: f64, text: &str, transform: Affine) {
        self.page_buffer
            .extend(format!("BT\n/F0 {} Tf\n", font_size).bytes());

        let transform = transform * Affine::FLIP_Y;
        let coeffs = transform.as_coeffs();

        write!(
            self.page_buffer,
            "{} {} {} {} {} {} ",
            coeffs[0], coeffs[1], coeffs[2], coeffs[3], coeffs[4], coeffs[5]
        )
        .unwrap();
        self.page_buffer.extend_from_slice(b"Tm (");
        for c in text.chars() {
            let data = format!("\\{:o}", c as u32);
            self.page_buffer.extend(data.bytes());
        }
        self.page_buffer.extend(b") Tj\n");
        self.page_buffer.extend(b"ET\n");
    }

    /// Move to a new page in the PDF document
    #[inline]
    pub fn add_page(&mut self, size: Size) {
        // Compress and write out the previous page if it exists
        if !self.page_buffer.is_empty() {
            self.end_page();
            self.page_buffer.clear();
        }

        self.fill_color = Vec::new();
        self.page_buffer
            .extend("/DeviceRGB cs /DeviceRGB CS\n1 j 1 J\n".bytes());
        self.page_size = Some(size);
    }

    pub fn move_to(&mut self, p: Point) {
        self.page_buffer
            .write_all(
                &[&p.x.to_string(), " ", &p.y.to_string(), " m\n"]
                    .concat()
                    .into_bytes(),
            )
            .unwrap();
    }

    pub fn line_to(&mut self, p: Point) {
        self.page_buffer
            .write_all(
                &[&p.x.to_string(), " ", &p.y.to_string(), " l\n"]
                    .concat()
                    .into_bytes(),
            )
            .unwrap();
    }

    pub fn cubic_to(&mut self, c1: Point, c2: Point, p: Point) {
        self.page_buffer
            .write_all(
                &[
                    &c1.x.to_string(),
                    " ",
                    &c1.y.to_string(),
                    " ",
                    &c2.x.to_string(),
                    " ",
                    &c2.y.to_string(),
                    " ",
                    &p.x.to_string(),
                    " ",
                    &p.y.to_string(),
                    " c\n",
                ]
                .concat()
                .into_bytes(),
            )
            .unwrap();
    }
    pub fn fill(&mut self) {
        self.page_buffer.write_all(b"f\n").unwrap();
    }

    pub fn close(&mut self) {
        self.page_buffer.write_all(b"h\n").unwrap();
    }

    fn end_page(&mut self) {
        let size = match self.page_size.take() {
            Some(size) => size,
            None => return, // no page started
        };
        let page_stream = if let Some(level) = self.compression {
            let compressed = deflate::deflate_bytes_zlib_conf(&self.page_buffer, level);
            let mut page = format!(
                "<< /Length {} /Filter [/FlateDecode] >>\nstream\n",
                compressed.len()
            )
            .into_bytes();
            page.extend_from_slice(&compressed);
            page.extend(b"endstream\n");
            page
        } else {
            let mut page = Vec::new();
            page.extend(format!("<< /Length {} >>\nstream\n", self.page_buffer.len()).bytes());
            page.extend(&self.page_buffer);
            page.extend(b"endstream\n");
            page
        };

        // Create the stream object for this page
        let stream_object_id = self.add_object(page_stream, false, false);

        // Create the page object, which describes settings for the whole page
        let mut page_object = b"<< /Type /Page\n \
            /Parent 2 0 R\n \
            /Resources <<\n"
            .to_vec();

        for (idx, _obj) in self
            .objects
            .iter()
            .enumerate()
            .filter(|&(_, o)| o.is_xobject)
        {
            write!(page_object, "/XObject {} 0 R ", idx + 1).unwrap();
        }

        write!(
            page_object,
            "  /Font <<\n   /F0 <<\n    /Type /Font\n    /Subtype /Type1\n    /BaseFont \
                     /Times-Roman\n    /Encoding /WinAnsiEncoding\n   >>\n  >>\n",
        )
        .unwrap();

        write!(
            page_object,
            " >>\n \
             /MediaBox [0 0 {} {}]\n \
             /Contents {} 0 R\n\
             >>\n",
            size.width, size.height, stream_object_id
        )
        .unwrap();
        self.add_object(page_object, true, false);
    }

    pub fn add_file(&mut self, file_str: &str) {
        let file_stream = if let Some(level) = self.compression {
            let compressed = deflate::deflate_bytes_zlib_conf(file_str.as_bytes(), level);
            let mut file = format!(
                "<< /Type /EmbeddedFile /Subtype /application#2Fjson /Length {} /Filter [/FlateDecode] >>\nstream\n",
                compressed.len()
            )
            .into_bytes();
            file.extend_from_slice(&compressed);
            file.extend(b"endstream\n");
            file
        } else {
            let mut file = Vec::new();
            let file_bytes = file_str.as_bytes();
            file.extend(
                format!(
                    "<< /Type /EmbeddedFile /Subtype /application#2Fjson /Length {} >>\nstream\n",
                    file_bytes.len()
                )
                .bytes(),
            );
            file.extend(file_bytes);
            file.extend(b"endstream\n");
            file
        };

        let file_obj = self.add_object(file_stream, false, false);
        self.file = Some(file_obj);
    }

    fn add_object(&mut self, data: Vec<u8>, is_page: bool, is_xobject: bool) -> usize {
        self.objects.push(PdfObject {
            contents: data,
            is_page,
            is_xobject,
            offset: None,
        });
        self.objects.len()
    }

    pub fn into_binary(self) -> Vec<u8> {
        let mut binary = Vec::new();
        self.write_to(&mut binary)
            .expect("Vec<u8> should not have IO issues.");
        binary
    }

    /// Write the in-memory PDF representation.
    ///
    /// This can be to disk, or to a Vec<u8>, or something else.
    pub fn write_to<W>(mut self, writer: W) -> io::Result<()>
    where
        W: Write,
    {
        let mut out = Counter::new(writer);
        out.write_all(b"%PDF-1.7\n%\xB5\xED\xAE\xFB\n")?;

        if !self.page_buffer.is_empty() {
            self.end_page();
        }

        // Write out each object
        for (idx, obj) in self.objects.iter_mut().enumerate().skip(2) {
            obj.offset = Some(out.pos());
            writeln!(out, "{} 0 obj", idx + 1)?;
            out.write_all(&obj.contents)?;
            out.write_all(b"endobj\n")?;
        }

        // Write out the page tree object
        self.objects[1].offset = Some(out.pos());
        out.write_all(b"2 0 obj\n")?;
        out.write_all(b"<< /Type /Pages\n")?;
        writeln!(
            out,
            "/Count {}",
            self.objects.iter().filter(|o| o.is_page).count()
        )?;
        out.write_all(b"/Kids [")?;
        for (idx, _obj) in self
            .objects
            .iter()
            .enumerate()
            .filter(|&(_, obj)| obj.is_page)
        {
            write!(out, "{} 0 R ", idx + 1)?;
        }
        out.write_all(b"] >>\nendobj\n")?;

        // Write out the catalog dictionary object
        self.objects[0].offset = Some(out.pos());
        let files = if let Some(file) = self.file {
            format!(
                "\n/Names << /EmbeddedFiles << /Names [ (sixeight.json) << /EF << /F {} 0 R >> /F (sixeight.json) /Type /F >> ] >> >>",
                file
            )
        } else {
            String::default()
        };

        write!(
            out,
            "1 0 obj\n<< /Type /Catalog{}\n/Pages 2 0 R >>\nendobj\n",
            files,
        )?;

        // Write the cross-reference table
        let startxref = out.pos();
        out.write_all(b"xref\n")?;
        writeln!(out, "0 {}", self.objects.len() + 1)?;
        out.write_all(b"0000000000 65535 f \n")?;

        for obj in &self.objects {
            writeln!(out, "{:010} 00000 n ", obj.offset.unwrap())?;
        }

        // Write the document trailer
        out.write_all(b"trailer\n")?;
        writeln!(out, "<< /Size {}", self.objects.len() + 1)?;
        out.write_all(b"/Root 1 0 R >>\n")?;

        // Write the offset to the xref table
        write!(out, "startxref\n{}\n", startxref)?;

        // Write the PDF EOF
        out.write_all(b"%%EOF")?;

        Ok(())
    }

    fn write_path(&mut self, path: &BezPath, transform: Affine) {
        let mut from = Point::new(0f64, 0f64);

        for event in (transform * path).elements() {
            match event {
                PathEl::MoveTo(to) => {
                    self.move_to(*to);
                    from = *to;
                }
                PathEl::LineTo(to) => {
                    self.line_to(*to);
                    from = *to;
                }
                PathEl::QuadTo(ctrl, to) => {
                    let ctrl1 = Affine::scale(2.0 / 3.0) * *ctrl
                        + (Affine::scale(1.0 / 3.0) * from).to_vec2();
                    let ctrl2 = Affine::scale(2.0 / 3.0) * *ctrl
                        + (Affine::scale(1.0 / 3.0) * *to).to_vec2();
                    self.cubic_to(ctrl1, ctrl2, *to);
                    from = *to;
                }
                PathEl::CurveTo(ctrl1, ctrl2, to) => {
                    self.cubic_to(*ctrl1, *ctrl2, *to);
                    from = *to;
                }
                PathEl::ClosePath => {
                    self.close();
                }
            }
        }
        self.fill();
    }

    pub fn write_stencil(
        &mut self,
        stencil: &Stencil,
        transform: Affine,
        stencils: &HashMap<Entity, Stencil>,
        stencil_maps: &HashMap<Entity, StencilMap>,
    ) {
        match stencil {
            Stencil::RawSvg(svg) => {
                self.write_path(&BezPath::from_svg(&svg.svg).unwrap(), transform);
            }
            Stencil::Path(path) => {
                self.write_path(&path.outline, transform);
            }
            Stencil::Text(text) => {
                self.write_text(text.font_size, &text.text, transform);
            }
            Stencil::Combine(CombineStencil(combine)) => {
                for stencil in combine {
                    self.write_stencil(stencil, transform, stencils, stencil_maps);
                }
            }
            Stencil::Translate(offset, child) => {
                self.write_stencil(
                    child,
                    transform * Affine::translate(*offset),
                    stencils,
                    stencil_maps,
                );
            }
        }
    }

    pub fn write_stencil_map(
        &mut self,
        stencil_map: &StencilMap,
        transform: Affine,
        stencils: &HashMap<Entity, Stencil>,
        stencil_maps: &HashMap<Entity, StencilMap>,
    ) {
        let transform = if let Some(translate) = stencil_map.translate {
            transform * Affine::translate(translate)
        } else {
            transform
        };

        for (child, translation) in stencil_map.get_sorted_children() {
            let child_transform = if let Some(translation) = translation {
                transform * Affine::translate(translation)
            } else {
                transform
            };
            if let Some(child) = stencils.get(&child) {
                self.write_stencil(child, child_transform, stencils, stencil_maps);
            }
            if let Some(child) = stencil_maps.get(&child) {
                self.write_stencil_map(child, child_transform, stencils, stencil_maps);
            }
        }
    }
}
