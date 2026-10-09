// This prevents the terminal swapning when run in windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
//! Desktop frontend: type an amount, read it in Turkish and in the other lira
//! era.

use std::sync::Arc;

use eframe::egui;
use try_conv::{Amount, Era, Outcome, interpret};

/// Application state: what the user typed and which era it is written in.
#[derive(Default)]
struct ConverterApp
{
    /// The typed text.
    input: String,
    /// The era `input` is written in.
    era: Era,
}

impl eframe::App for ConverterApp
{
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame)
    {
        egui::CentralPanel::default().show(ui, |ui| {
            // A long amount makes the output taller than the window. Without
            // this scroll area the second era block and the footer notes are
            // cut off, and no scrollbar appears.
            egui::ScrollArea::vertical().show(ui, |ui| self.contents(ui));
        });
    }
}

impl ConverterApp
{
    /// Draws the era selector, the text field, the reading with the other
    /// era's line and the footer notes.
    fn contents(&mut self, ui: &mut egui::Ui)
    {
        ui.horizontal(|ui| {
            ui.label("Girdi dönemi:");
            ui.radio_value(
                &mut self.era,
                Era::OldTrl,
                "Eski TL (TRL, 2005 öncesi)",
            );
            ui.radio_value(&mut self.era, Era::NewTry, "Yeni TL (TRY, 2009-)");
        });
        ui.horizontal(|ui| {
            ui.label("Tutar ya da okunuşu:");
            ui.add(
                egui::TextEdit::singleline(&mut self.input)
                    .hint_text("1.250.000,75")
                    .desired_width(f32::INFINITY),
            );
        });
        ui.separator();
        match interpret(&self.input)
        {
            Ok(Outcome::Idle) =>
            {
                ui.weak("Bir tutar ya da okunuş yazın.");
            },
            Ok(Outcome::FromNumber(amount) | Outcome::FromWords(amount)) =>
            {
                self.show_amount(ui, &amount);
            },
            Err(error) =>
            {
                ui.colored_label(egui::Color32::RED, error.to_string());
            },
        }
        ui.separator();
        #[rustfmt::skip]
        ui.small(
            "Not: 1 Ocak 2005'te 6 sıfır atıldı - 1.000.000 eski TL = 1 yeni TL."
        );
        #[rustfmt::skip]
        ui.small(
            "2005-2008 arası \"Yeni Türk Lirası (YTL)\" adı kullanıldı; değeri TL ile aynıdır.",
        );
    }

    /// Draws the reading of the typed amount and of its other-era equivalent.
    fn show_amount(&self, ui: &mut egui::Ui, amount: &Amount)
    {
        // The era the user selected is the one they typed, so the reading says
        // it once and only the other era gets a block: repeating the selected
        // era would print this same reading a second time.
        match amount.to_words_lira()
        {
            Ok(words) =>
            {
                ui.horizontal_wrapped(|ui| {
                    ui.label(egui::RichText::new("Okunuşu:").strong());
                    ui.label(egui::RichText::new(words.as_str()));
                });
            },
            Err(error) =>
            {
                ui.colored_label(egui::Color32::RED, error.to_string());
            },
        }

        ui.add_space(6.0);

        match amount.shifted_by_million(self.era.conversion_multiplies())
        {
            Ok(other) =>
            {
                draw_era_line(ui, self.era.other_era(), &other);
            },
            Err(error) =>
            {
                ui.colored_label(egui::Color32::RED, error.to_string());
            },
        }
    }
}

/// Draws one era's digits and reading.
fn draw_era_line(ui: &mut egui::Ui, era: Era, amount: &Amount)
{
    // Both lines wrap: the ceiling can produce a 408-character digits line
    // (306 digits plus separators), which no window shows on one line.
    ui.horizontal_wrapped(|ui| {
        ui.label(
            egui::RichText::new(format!("{}:", era))
                .strong()
                .size(17.0),
        );
        ui.label(
            egui::RichText::new(amount.grouped())
                .size(15.5)
                .monospace(),
        );
    });

    match amount.to_words_lira()
    {
        Ok(words) =>
        {
            ui.add(
                egui::Label::new(
                    egui::RichText::new(format!("  okunuşu: {words}"))
                        .weak()
                        .size(15.0)
                        .italics(),
                )
                .wrap(),
            );
        },
        Err(error) =>
        {
            ui.colored_label(egui::Color32::RED, error.to_string());
        },
    }
}

/// Sets the base text sizes and the scrollbar style, in logical points.
///
/// egui's defaults (13 pt body, 9 pt small) are tuned for dense desktop
/// displays and read as small at the 125 % scaling.
///
/// They are raised by one modest step; the sizes stay in logical points, so the
/// window's own scale factor still converts them and every display density
/// stays proportional. Ctrl `+` / Ctrl `-` remain the per-machine override.
fn apply_text_sizes(ctx: &egui::Context)
{
    ctx.all_styles_mut(|style| {
        for (text_style, points) in [
            (egui::TextStyle::Body, 16.0),
            (egui::TextStyle::Button, 16.0),
            (egui::TextStyle::Small, 12.0),
        ]
        {
            style
                .text_styles
                .insert(text_style, egui::FontId::proportional(points));
        }
        // egui's default scrollbar floats over the content and appears only
        // while the pointer is inside the scroll area, so a clipped page looks
        // like a dead end. A solid bar keeps its own column and shows whenever
        // the content is taller than the window.
        style.spacing.scroll = egui::style::ScrollStyle::solid();
    });
}

/// Loads the embedded icon when the asset exists and decodes successfully.
fn load_icon() -> Option<Arc<egui::IconData>>
{
    #[cfg(has_icon)]
    {
        /// Decodes icon bytes, returning `None` when they are not a valid PNG.
        fn decode_icon(bytes: &[u8]) -> Option<Arc<egui::IconData>>
        {
            eframe::icon_data::from_png_bytes(bytes)
                .ok()
                .map(Arc::new)
        }

        decode_icon(include_bytes!("../assets/icon.png"))
    }
    #[cfg(not(has_icon))]
    {
        None
    }
}

/// Opens the window, using the bundled icon when one is available.
fn main() -> eframe::Result
{
    let viewport = egui::ViewportBuilder::default()
        .with_title("TL / TRY Okunuş Çevirici")
        .with_inner_size([760.0, 360.0])
        // Logical points: 760 fits the longest reading, and 520 keeps the
        // era selector row (about 480 wide) fully visible at the minimum.
        .with_min_inner_size([520.0, 260.0]);
    let viewport = match load_icon()
    {
        Some(icon) => viewport.with_icon(icon),
        None => viewport,
    };

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        env!("CARGO_PKG_NAME"),
        options,
        Box::new(|cc| {
            apply_text_sizes(&cc.egui_ctx);
            Ok(Box::new(ConverterApp::default()))
        }),
    )
}

#[cfg(test)]
mod tests
{
    use super::{decode_icon, load_icon};

    /// Keeps malformed embedded image data from preventing app startup.
    #[test]
    fn ignores_invalid_icon_data()
    {
        assert!(decode_icon(b"not a png").is_none());
    }

    /// Embeds and decodes the icon when the asset is present at build time.
    #[cfg(has_icon)]
    #[test]
    fn loads_the_bundled_icon()
    {
        assert!(load_icon().is_some());
    }

    /// Builds and starts with no custom icon when the asset is absent.
    #[cfg(not(has_icon))]
    #[test]
    fn omits_a_missing_icon()
    {
        assert!(load_icon().is_none());
    }
}
