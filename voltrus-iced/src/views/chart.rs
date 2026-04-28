use iced::widget::{button, column, row, text, Space};
use iced::{Alignment, Element, Length, Padding};
use plotters::prelude::*;
use plotters_iced::{Chart, ChartWidget};

use crate::app::{App, Message};
use crate::models;
use crate::theme;

// ---------------------------------------------------------------------------
// Chart data structure
// ---------------------------------------------------------------------------

pub struct HistoryChart {
    points: Vec<models::DataPoint>,
    warn_val: f64,
    crit_val: f64,
    unit: String,
}

impl HistoryChart {
    pub fn new(
        points: Vec<models::DataPoint>,
        warn_val: f64,
        crit_val: f64,
        unit: String,
    ) -> Self {
        Self {
            points,
            warn_val,
            crit_val,
            unit,
        }
    }
}

impl Chart<Message> for HistoryChart {
    type State = ();

    fn build_chart<DB: DrawingBackend>(&self, _state: &Self::State, mut builder: ChartBuilder<DB>) {
        if self.points.is_empty() {
            return;
        }

        let min_ts = self.points.iter().map(|p| p.timestamp).min().unwrap_or(0);
        let max_ts = self.points.iter().map(|p| p.timestamp).max().unwrap_or(1).max(min_ts + 1);
        let min_val = self.points.iter().map(|p| p.value).filter(|v| v.is_finite()).fold(f64::INFINITY, f64::min);
        let max_val = self.points.iter().map(|p| p.value).filter(|v| v.is_finite()).fold(f64::NEG_INFINITY, f64::max);

        let val_range = max_val - min_val;
        let padding = val_range.max(1.0) * 0.1;
        let y_min = (min_val - padding).min(self.warn_val.min(self.crit_val)).max(0.0);
        let y_max = (max_val + padding).max(self.crit_val.max(self.warn_val).max(min_val + 1.0));

        let unit_label = if self.unit.is_empty() {
            "Value".to_string()
        } else {
            self.unit.clone()
        };

        let mut chart = builder
            .x_label_area_size(40)
            .y_label_area_size(60)
            .margin(10)
            .caption("", ("sans-serif", 16).into_font())
            .build_cartesian_2d(min_ts..max_ts, y_min..y_max)
            .unwrap();

        chart
            .configure_mesh()
            .x_labels(6)
            .x_label_formatter(&|ts: &i64| format_time(*ts))
            .y_labels(6)
            .y_label_formatter(&|val: &f64| format_unit_value(*val, &unit_label))
            .x_desc("Time")
            .y_desc(&unit_label)
            .label_style(("sans-serif", 10).into_font())
            .axis_style(ShapeStyle::from(&RGBColor(120, 120, 120)).stroke_width(1))
            .light_line_style(ShapeStyle::from(&RGBColor(40, 40, 45)).stroke_width(1))
            .draw()
            .unwrap();

        // Main data line (cyan)
        chart
            .draw_series(
                LineSeries::new(
                    self.points.iter().map(|p| (p.timestamp, p.value)),
                    ShapeStyle::from(&RGBColor(34, 197, 94)).stroke_width(2),
                ),
            )
            .unwrap()
            .label("Value")
            .legend(|(x, y)| {
                Rectangle::new([(x, y - 5), (x + 20, y + 5)], RGBColor(34, 197, 94).filled())
            });

        // Warning threshold line
        if self.warn_val.is_finite() && self.warn_val > y_min && self.warn_val < y_max {
            chart
                .draw_series(
                    LineSeries::new(
                        [(min_ts, self.warn_val), (max_ts, self.warn_val)],
                        ShapeStyle::from(&RGBColor(245, 158, 11)).stroke_width(1),
                    ),
                )
                .unwrap()
                .label("Warning")
                .legend(|(x, y)| {
                    let mut seg = Vec::new();
                    for i in 0..10 {
                        if i % 2 == 0 {
                            seg.push((x + i as i32 * 2, y));
                        } else {
                            seg.push((x + i as i32 * 2 - 1, y));
                        }
                    }
                    PathElement::new(seg, RGBColor(245, 158, 11))
                });
        }

        // Critical threshold line
        if self.crit_val.is_finite() && self.crit_val > y_min && self.crit_val < y_max {
            chart
                .draw_series(
                    LineSeries::new(
                        [(min_ts, self.crit_val), (max_ts, self.crit_val)],
                        ShapeStyle::from(&RGBColor(239, 68, 68)).stroke_width(1),
                    ),
                )
                .unwrap()
                .label("Critical")
                .legend(|(x, y)| {
                    let mut seg = Vec::new();
                    for i in 0..10 {
                        if i % 2 == 0 {
                            seg.push((x + i as i32 * 2, y));
                        } else {
                            seg.push((x + i as i32 * 2 - 1, y));
                        }
                    }
                    PathElement::new(seg, RGBColor(239, 68, 68))
                });
        }

        chart
            .configure_series_labels()
            .position(SeriesLabelPosition::UpperRight)
            .label_font(("sans-serif", 10))
            .background_style(RGBColor(17, 17, 19).filled())
            .border_style(ShapeStyle::from(&RGBColor(39, 39, 42)))
            .draw()
            .unwrap();
    }
}

// ---------------------------------------------------------------------------
// Time formatting helpers
// ---------------------------------------------------------------------------

fn format_time(timestamp_ms: i64) -> String {
    let secs = timestamp_ms / 1000;
    let hours = ((secs % 86400) / 3600).rem_euclid(24) as u32;
    let mins = ((secs % 3600) / 60).rem_euclid(60) as u32;
    format!("{:02}:{:02}", hours, mins)
}

fn format_unit_value(val: f64, unit: &str) -> String {
    if unit.is_empty() {
        format!("{:.1}", val)
    } else {
        format!("{:.1} {}", val, unit)
    }
}

// ---------------------------------------------------------------------------
// View
// ---------------------------------------------------------------------------

const RANGES: &[&str] = &["30m", "1h", "6h", "24h"];

pub fn view(app: &App) -> Element<'_, Message> {
    let sensor_info = app
        .data_sources
        .iter()
        .find(|s| Some(s.id) == app.selected_sensor);

    let sensor_name = sensor_info
        .map(|s| s.name.as_str())
        .unwrap_or("Unknown Sensor");
    let warn_val = sensor_info.map(|s| s.warn_val).unwrap_or(f64::NAN);
    let crit_val = sensor_info.map(|s| s.crit_val).unwrap_or(f64::NAN);
    let unit = sensor_info
        .map(|s| s.unit.clone())
        .unwrap_or_default();

    // --- Header ---
    let back_btn = button(
        text("<< Back")
            .size(12)
            .font(iced::Font::DEFAULT),
    )
    .padding([6, 12])
    .on_press(Message::BackToGrid)
    .style(theme::header_btn_style());

    let title = text(sensor_name)
        .size(18)
        .color(theme::TEXT)
        .font(iced::Font::DEFAULT);

    let header = row![back_btn, title]
        .spacing(12)
        .align_y(Alignment::Center)
        .padding(Padding::new(12.0).left(20.0).right(20.0));

    // --- Range tabs ---
    let range_buttons: Vec<Element<'_, Message>> = RANGES
        .iter()
        .map(|range| {
            let is_active = app.history_range == *range;
            button(
                text(*range)
                    .size(12)
                    .font(iced::Font::DEFAULT),
            )
            .padding([8, 16])
            .style(theme::tab_button_style(is_active))
            .on_press(Message::SetRange(range.to_string()))
            .into()
        })
        .collect();

    let range_row = row(range_buttons)
        .spacing(0)
        .padding(Padding::new(0.0).left(20.0).right(20.0).bottom(8.0));

    // --- Content ---
    let content: Element<'_, Message> = if app.history_loading {
        column![
            Space::with_height(100),
            text("Loading history...").size(14).color(theme::TEXT_MUTED),
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(iced::alignment::Horizontal::Center)
        .into()
    } else if app.history_points.is_empty() {
        column![
            Space::with_height(100),
            text("No data available").size(14).color(theme::TEXT_MUTED),
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(iced::alignment::Horizontal::Center)
        .into()
    } else {
        let chart = HistoryChart::new(
            app.history_points.clone(),
            warn_val,
            crit_val,
            unit,
        );
        ChartWidget::new(chart)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    };

    column![header, range_row, content]
        .width(Length::Fill)
        .height(Length::Fill)
        .spacing(0)
        .padding(Padding::new(0.0).bottom(12.0))
        .into()
}
