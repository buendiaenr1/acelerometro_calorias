use chrono::{NaiveDate, NaiveTime, NaiveDateTime};
use std::fs::File;
use std::io::{BufReader, BufRead, Write};
use plotly::common::{Mode, Line};
use plotly::layout::{Layout, Axis};
use plotly::{Plot, Scatter, Histogram};
use std::process::Command;
use egui::{CentralPanel, Context, Frame, Ui, Window};
use eframe::egui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Iniciar la aplicación con interfaz gráfica
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Gasto Energético Promedio   Enrique Buendia Lozada BUAP",
        options,
        Box::new(|_cc| Ok(Box::new(MyApp::default()))),
    );

    Ok(())
}

struct MyApp {
    dates: Vec<NaiveDateTime>,
    accel_magnitudes: Vec<f64>,
    peso: f64,
    calorias_totales: f64,
    calorias_totales2: f64,
    magnitud_promedio: f64,
    met: f64,
    met2: f64,
    tiempo_total_horas: f64,
    frecuencia_muestreo_hz: f64,
    intervalo_promedio_segundos: f64,
    numero_intervalos: usize,
    magnitudes_mayores_o_iguales_a_20: usize, // Nueva variable para contar magnitudes >= 20
}

impl Default for MyApp {
    fn default() -> Self {
        let file = File::open("accdata.csv").expect("No se pudo abrir el archivo");
        let reader = BufReader::new(file);

        let mut dates: Vec<NaiveDateTime> = Vec::new();
        let mut accel_magnitudes: Vec<f64> = Vec::new();

        for (i, line) in reader.lines().enumerate() {
            let line = line.expect("Error al leer línea");
            if i == 0 {
                continue;
            }

            let columns: Vec<&str> = line.split(',').collect();
            if columns.len() < 5 {
                eprintln!("Línea mal formada: {}", line);
                continue;
            }

            let date_str = columns[0].trim();
            let time_str = columns[1].trim();
            let accel_x: f64 = columns[2].trim().parse().unwrap_or_default();
            let accel_y: f64 = columns[3].trim().parse().unwrap_or_default();
            let accel_z: f64 = columns[4].trim().parse().unwrap_or_default();

            let parsed_date = NaiveDate::parse_from_str(date_str, "%d/%m/%Y").unwrap_or_default();
            let parsed_time = NaiveTime::parse_from_str(time_str, "%H:%M:%S").unwrap_or_default();
            let datetime = NaiveDateTime::new(parsed_date, parsed_time);

            dates.push(datetime);
            let magnitude = (accel_x.powi(2) + accel_y.powi(2) + accel_z.powi(2)).sqrt();
            accel_magnitudes.push(magnitude);
        }

        // Contar magnitudes iguales o mayores que 20
        let magnitudes_mayores_o_iguales_a_20 = accel_magnitudes.iter().filter(|&&mag| mag >= 20.0).count();

        // Guardar las magnitudes en un archivo CSV
        //let mut mag_file = File::create("mag_accel.csv").expect("No se pudo crear el archivo");
        //for magnitude in &accel_magnitudes {
        //    writeln!(mag_file, "{}", magnitude).expect("Error al escribir en el archivo");
        //}

        dates.sort();

        let mut intervalos_segundos = Vec::new();
        for i in 1..dates.len() {
            let prev = dates[i - 1];
            let curr = dates[i];
            let diff = (curr - prev).num_seconds();
            intervalos_segundos.push(diff as f64);
        }

        let numero_intervalos = intervalos_segundos.len();
        let intervalo_promedio_segundos: f64 = intervalos_segundos.iter().sum::<f64>() / numero_intervalos as f64;
        let frecuencia_muestreo_hz = 1.0 / intervalo_promedio_segundos;

        let magnitud_promedio: f64 = accel_magnitudes.iter().sum::<f64>() / accel_magnitudes.len() as f64;
        //let met = if magnitud_promedio < 1.5 {
        //    2.0
        //} else if magnitud_promedio < 3.0 {
        //    3.5
        //} else {
        //    7.0
        //};
        let met = if magnitud_promedio < 3.0 {
            2.0 // Actividad ligera (<3.0 METs)
        } else if magnitud_promedio < 6.0 {
            4.0 // Actividad moderada (3.0-5.9 METs)
        } else {
            7.0 // Actividad vigorosa (≥6.0 METs)
        };
        let met2 = if magnitud_promedio < 4.9 {
            2.0 // Actividad ligera (<4.9 METs)
        } else if magnitud_promedio < 6.8 {
            4.0 // Actividad moderada (4.9-6.8 METs)
        } else {
            7.0 // Actividad vigorosa (≥6.8 METs)
        };

        let tiempo_total_segundos: f64 = intervalos_segundos.iter().sum();
        let tiempo_total_horas = tiempo_total_segundos / 3600.0;

        // Graficar histograma de las magnitudes
        let trace_hist = Histogram::new(accel_magnitudes.clone()).name("Magnitudes de Aceleración");
        let layout_hist = Layout::new()
            .title("Histograma de Magnitudes de Aceleración")
            .x_axis(Axis::new().title("Magnitud"))
            .y_axis(Axis::new().title("Frecuencia"));

        let mut plot_hist = Plot::new();
        plot_hist.add_trace(trace_hist);
        plot_hist.set_layout(layout_hist);
        plot_hist.show();

        // Graficar diagrama de dispersión con línea del promedio
        let x_values: Vec<usize> = (0..accel_magnitudes.len()).collect();
        let trace_scatter = Scatter::new(x_values.clone(), accel_magnitudes.clone())
            .name("Magnitudes de Aceleración")
            .mode(Mode::Markers);

        let trace_promedio = Scatter::new(x_values.clone(), vec![magnitud_promedio; x_values.len()])
            .name("Promedio")
            .mode(Mode::Lines)
            .line(Line::new().color("red"));

        let layout_scatter = Layout::new()
            .title("Diagrama de Dispersión de Magnitudes de Aceleración")
            .x_axis(Axis::new().title("Índice de Muestra"))
            .y_axis(Axis::new().title("Magnitud de Aceleración"));

        let mut plot_scatter = Plot::new();
        plot_scatter.add_trace(trace_scatter);
        plot_scatter.add_trace(trace_promedio); // Agregar la línea del promedio
        plot_scatter.set_layout(layout_scatter);
        plot_scatter.show();

        MyApp {
            dates,
            accel_magnitudes,
            peso: 70.0,
            calorias_totales: 0.0,
            calorias_totales2: 0.0,
            magnitud_promedio,
            met,
            met2,
            tiempo_total_horas,
            frecuencia_muestreo_hz,
            intervalo_promedio_segundos,
            numero_intervalos,
            magnitudes_mayores_o_iguales_a_20, // Asignar el conteo a la estructura
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Gasto Energético Promedio");

            ui.label("Ingrese su peso en kg:");
            ui.add(egui::DragValue::new(&mut self.peso).speed(0.1));

            if ui.button("Calcular").clicked() {
                self.calorias_totales = self.met * self.peso * self.tiempo_total_horas;
                self.calorias_totales2 = self.met2 * self.peso * self.tiempo_total_horas;
            }

            ui.separator();
            ui.label(format!("Número de intervalos: {}", self.numero_intervalos));
            ui.label(format!(
                "Promedio de tiempo entre intervalos: {:.6} segundos",
                self.intervalo_promedio_segundos
            ));
            ui.label(format!("Frecuencia de muestreo (Hz): {:.6}", self.frecuencia_muestreo_hz));
            ui.label(format!(
                "Magnitud promedio de la aceleración: {:.6}",
                self.magnitud_promedio
            ));
            ui.label(format!("MET asignado: {:.1}", self.met));
            ui.label(format!("MET2 asignado: {:.1}", self.met2));
            ui.label(format!("Tiempo total de actividad: {:.2} horas", self.tiempo_total_horas));
            ui.label(format!("Calorías totales quemadas: {:.2} kcal  calorias_totales=MET*peso*tiempo_total_en_horas", self.calorias_totales));
            ui.label(format!("Calorías totales quemadas2: {:.2} kcal", self.calorias_totales2));

            // Mostrar el conteo de magnitudes >= 20
            ui.label(format!(
                "Magnitudes iguales o mayores que 20: {}",
                self.magnitudes_mayores_o_iguales_a_20
            ));

            // Botón para guardar las magnitudes en mag.csv
            if ui.button("Guardar Magnitudes en CSV").clicked() {
                let mut mag_csv_file = File::create("mag.csv").expect("No se pudo crear el archivo mag.csv");
                writeln!(mag_csv_file, "Magnitud").expect("Error al escribir encabezado en mag.csv");
                for magnitude in &self.accel_magnitudes {
                    writeln!(mag_csv_file, "{}", magnitude).expect("Error al escribir en mag.csv");
                }
                ui.label("¡Magnitudes guardadas en mag.csv!");
            }
        });
    }
}