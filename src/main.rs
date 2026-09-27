//! Desktop frontend: type an amount, read it in Turkish and in the other lira
//! era.

use eframe::egui;
use try_trl_conv::{
    Amount,
    Era,
    Outcome,
    conversion_multiplies,
    interpret,
    other_era,
};

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
        egui::CentralPanel::default().show(ui, |ui| self.contents(ui));
    }
}

impl ConverterApp
{
    /// Draws the era selector, the text field, both era lines and the footer
    /// notes.
    fn contents(&mut self, ui: &mut egui::Ui)
    {
        ui.horizontal(|ui| {
            ui.label("Girdi dönemi:");
            ui.radio_value(
                &mut self.era,
                Era::OldTrl,
                "Eski TL (TRL, 2005 öncesi)",
            );
            ui.radio_value(&mut self.era, Era::NewTry, "Yeni TL (TRY, 2009–)");
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
        ui.small(
            "Not: 1 Ocak 2005'te 6 sıfır atıldı — 1.000.000 eski TL = 1 yeni \
             TL.",
        );
        ui.small(
            "2005–2008 arası \"Yeni Türk Lirası (YTL)\" adı kullanıldı; \
             değeri TL ile aynıdır.",
        );
    }

    /// Draws the reading of the typed amount and of its other-era equivalent.
    fn show_amount(&self, ui: &mut egui::Ui, amount: &Amount)
    {
        match amount.to_words_lira()
        {
            Ok(words) =>
            {
                ui.add(egui::Label::new(format!("Okunuşu: {words}")).wrap());
            },
            Err(error) =>
            {
                ui.colored_label(egui::Color32::RED, error.to_string());
            },
        }
        ui.add_space(6.0);
        draw_era_line(ui, self.era, amount);
        match amount.shifted_by_million(conversion_multiplies(self.era))
        {
            Ok(other) => draw_era_line(ui, other_era(self.era), &other),
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
    // The spec keeps the digits line on one line and lets only the reading
    // wrap.
    ui.add(
        egui::Label::new(format!("{}: {}", era_label(era), amount.grouped()))
            .wrap_mode(egui::TextWrapMode::Extend),
    );
    match amount.to_words_lira()
    {
        Ok(words) =>
        {
            ui.add(egui::Label::new(format!("  okunuşu: {words}")).wrap());
        },
        Err(error) =>
        {
            ui.colored_label(egui::Color32::RED, error.to_string());
        },
    }
}

/// The Turkish label of an era.
const fn era_label(era: Era) -> &'static str
{
    match era
    {
        Era::OldTrl => "Eski TL (TRL)",
        Era::NewTry => "Yeni TL (TRY)",
    }
}

/// Sets the base text sizes, in logical points.
///
/// egui's defaults (13 pt body, 9 pt small) are tuned for dense desktop
/// displays and read as small at the 125 % scaling this app was reported on.
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
    });
}

/// Opens the window.
fn main() -> eframe::Result
{
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("TL / TRY Okunuş Çevirici")
            .with_inner_size([760.0, 360.0])
            .with_min_inner_size([420.0, 260.0]),
        ..Default::default()
    };
    eframe::run_native(
        "try_trl_conv",
        options,
        Box::new(|cc| {
            apply_text_sizes(&cc.egui_ctx);
            Ok(Box::new(ConverterApp::default()))
        }),
    )
}
