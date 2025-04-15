use chrono::{NaiveDate, NaiveTime, NaiveDateTime};
use std::fs::File;
use std::io::{BufReader, BufRead};
use plotly::common::{Mode, Line};
use plotly::layout::{Layout, Axis};
use plotly::{Plot, Scatter};
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // limpiar
    Command::new("cmd")
            .args(&["/C", "cls"])
            .status()
            .expect("Error al ejecutar el comando cls");

    println!(" \n\n\n BUAP 2025: Enrique Buendia Lozada");
    println!(" Gasto energético promedio usando medidas de acelerómetro ");
    println!(" Formato del archivo a usar: accdata.csv ");
    println!(" Date	        Time	    accel_x	        accel_y	        accel_z");
    println!(" 03/09/2022	23:35:16	-1.838746905	3.543418407	    9.126696587\n\n");

    // Abrir el archivo CSV
    let file = File::open("accdata.csv")?;
    let reader = BufReader::new(file);

    // Variables para almacenar los datos
    let mut dates: Vec<NaiveDateTime> = Vec::new();
    let mut accel_magnitudes: Vec<f64> = Vec::new();

    // Leer el archivo línea por línea
    for (i, line) in reader.lines().enumerate() {
        let line = line?;
        if i == 0 {
            // Ignorar la primera línea (encabezados)
            continue;
        }

        // Dividir la línea en columnas
        let columns: Vec<&str> = line.split(',').collect();
        if columns.len() < 5 {
            eprintln!("Línea mal formada: {}", line);
            continue;
        }

        // Extraer las columnas Date, Time, accel_x, accel_y, accel_z
        let date_str = columns[0].trim();
        let time_str = columns[1].trim();
        let accel_x: f64 = columns[2].trim().parse().unwrap_or_default();
        let accel_y: f64 = columns[3].trim().parse().unwrap_or_default();
        let accel_z: f64 = columns[4].trim().parse().unwrap_or_default();

        // Parsear la fecha y hora
        let parsed_date = NaiveDate::parse_from_str(date_str, "%d/%m/%Y").unwrap_or_default();
        let parsed_time = NaiveTime::parse_from_str(time_str, "%H:%M:%S").unwrap_or_default();
        let datetime = NaiveDateTime::new(parsed_date, parsed_time);

        // Almacenar el objeto NaiveDateTime
        dates.push(datetime);

        // Calcular la magnitud de la aceleración
        let magnitude = (accel_x.powi(2) + accel_y.powi(2) + accel_z.powi(2)).sqrt();
        accel_magnitudes.push(magnitude);
    }

    // Ordenar los datos por fecha y hora
    dates.sort();

    // Calcular los intervalos entre muestras consecutivas
    let mut intervalos_segundos = Vec::new();
    for i in 1..dates.len() {
        let prev = dates[i - 1];
        let curr = dates[i];

        // Calcular la diferencia en segundos entre dos fechas
        let diff = (curr - prev).num_seconds(); // Esto ya es un i64
        intervalos_segundos.push(diff as f64);
    }

    // Mostrar los intervalos de tiempo
    //println!("Intervalos de tiempo entre medidas:");
    //for (i, intervalo) in intervalos_segundos.iter().enumerate() {
    //    println!(
    //        "Entre {} y {}: {:.2} segundos",
    //        dates[i].format("%H:%M:%S"),
    //        dates[i + 1].format("%H:%M:%S"),
    //        intervalo
    //    );
    //}

    // Calcular el número de intervalos
    let numero_intervalos = intervalos_segundos.len();

    // Calcular el promedio de los intervalos
    let intervalo_promedio_segundos: f64 = intervalos_segundos.iter().sum::<f64>() / numero_intervalos as f64;

    // Calcular la frecuencia de muestreo (Hz)
    let frecuencia_muestreo_hz = 1.0 / intervalo_promedio_segundos;

    // Calcular la magnitud promedio de la aceleración
    let magnitud_promedio: f64 = accel_magnitudes.iter().sum::<f64>() / accel_magnitudes.len() as f64;

    // Asignar MET según la magnitud promedio
    let met = if magnitud_promedio < 1.5 {
        2.0 // Actividad ligera
    } else if magnitud_promedio < 3.0 {
        3.5 // Actividad moderada
    } else {
        7.0 // Actividad intensa
    };

    // Preguntar al usuario su peso
    let mut peso_input = String::new();
    println!("Ingrese su peso en kg:");
    std::io::stdin().read_line(&mut peso_input)?;
    let peso: f64 = peso_input.trim().parse().unwrap_or_default();

    // Tiempo total de actividad en horas
    let tiempo_total_segundos: f64 = intervalos_segundos.iter().sum();
    let tiempo_total_horas = tiempo_total_segundos / 3600.0;

    // Calcular el consumo de calorías
    let calorias_totales = met * peso * tiempo_total_horas;

    // Mostrar resultados
    println!("\nResumen:");
    println!("Número de intervalos: {}", numero_intervalos);
    println!(
        "Promedio de tiempo entre intervalos: {:.6} segundos",
        intervalo_promedio_segundos
    );
    println!("Frecuencia de muestreo (Hz): {:.6}", frecuencia_muestreo_hz);
    println!("Magnitud promedio de la aceleración: {:.6}", magnitud_promedio);
    println!("MET asignado: {:.1}", met);
    println!("Tiempo total de actividad: {:.2} horas", tiempo_total_horas);
    println!("Calorías totales quemadas: {:.2} kcal\n\n", calorias_totales);

    // Crear gráfica de líneas para la magnitud de la aceleración
    let x_values: Vec<usize> = (0..accel_magnitudes.len()).collect();
    let trace_data = Scatter::new(x_values.clone(), accel_magnitudes)
        .name("Magnitud de Aceleración")
        .mode(Mode::LinesMarkers);

    // Agregar una línea horizontal para el promedio
    let trace_promedio = Scatter::new(x_values.clone(), vec![magnitud_promedio; x_values.len()])
        .name("Promedio")
        .mode(Mode::Lines)
        .line(Line::new().color("red"));

    let layout = Layout::new()
        .title("Magnitud de Aceleración sobre el Tiempo")
        .x_axis(Axis::new().title("Índice de Muestra"))
        .y_axis(Axis::new().title("Magnitud de Aceleración"));

    let mut plot = Plot::new();
    plot.add_trace(trace_data);
    plot.add_trace(trace_promedio); // Agregar la línea del promedio
    plot.set_layout(layout);

    // Guardar la gráfica como HTML y abrirla en el navegador
    plot.show();

    // Mantener la consola abierta después de salir
    Command::new("cmd")
            .args(&["/C", "cmd /k"])
            .status()
            .expect("Error al ejecutar el comando cmd /k");

    Ok(())
}