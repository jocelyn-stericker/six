import { Song, update } from "./store";
import { fileOpen } from "browser-nativefs";

export async function loadPdf(): Promise<{ error: string } | { song: Song }> {
  const pdf = await fileOpen({
    description: "Six Eight PDF",
    extensions: ["pdf"],
    mimeTypes: ["application/pdf"],
  });
  try {
    const raw = await new Promise<string | ArrayBuffer | null>(
      (resolve, reject) => {
        const reader = new FileReader();
        reader.readAsDataURL(pdf);
        reader.onerror = error => reject(error);
        reader.onload = () => {
          resolve(reader.result);
        };
      },
    );
    const {
      PDFDocument,
      PDFName,
      decodePDFRawStream,
      PDFRawStream,
    } = await import("pdf-lib");
    // @ts-ignore
    const doc = await PDFDocument.load(raw);
    const fileRef = doc?.catalog
      ?.get(PDFName.of("Names"))
      // @ts-ignore -- TODO
      ?.get(PDFName.of("EmbeddedFiles"))
      ?.get(PDFName.of("Names"))
      ?.get(1)
      ?.get(PDFName.of("EF"))
      ?.get(PDFName.of("F"));
    if (!fileRef) {
      return {
        error: "This does not appear to be a PDF generated by Six Eight.",
      };
    }
    // @ts-ignore -- TODO
    const rawStream = doc.context.lookupMaybe(fileRef, PDFRawStream);
    // @ts-ignore -- TODO
    const parsed = decodePDFRawStream(rawStream).decode();
    const song = JSON.parse(new TextDecoder("utf-8").decode(parsed));
    if (typeof song.v !== "number") {
      return { error: "Unsupported song version." };
    }
    return { song: update(song) };
  } catch (err) {
    console.warn(err);
    return { error: "Failed to process pdf." };
  }
}

export async function savePdf(
  filename: string,
  pdfBase64: string,
): Promise<void> {
  const a = document.createElement("a");
  a.href = `data:application/pdf;base64,${pdfBase64}`;
  a.download = `${filename}.pdf`;
  a.click();
}
