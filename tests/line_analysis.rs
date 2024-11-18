use {
    bacon::*,
    termimad::crossterm::style::Stylize,
};

/// Check we reproduce all the line analyses from json files in line_analysis directory
#[test]
fn line_analysis() {

    todo!();

    //let dir = file!().strip_suffix(".rs").unwrap();
    //for entry in std::fs::read_dir(dir).unwrap() {
    //    let entry = entry.unwrap();
    //    let path = entry.path();
    //    let (Some(name), Some(extension)) = (path.file_name(), path.extension()) else {
    //        continue;
    //    };
    //    if extension != "json" {
    //        continue;
    //    }
    //    let file = std::fs::File::open(&path).unwrap();
    //    let reader = std::io::BufReader::new(file);
    //    let export: AnalysisExport = serde_json::from_reader(reader).unwrap();
    //    let analyzer = export.analyzer;
    //    for line_entry in export.lines {
    //        // checking that we reproduce the same analysis
    //        let analysis = analyzer.analyze_line(&line_entry.line);
    //        if analysis != line_entry.analysis {
    //            println!(
    //                "Wrong analysis in {} with analyzer {:?} for {:#?}",
    //                name.to_string_lossy().to_string().blue(),
    //                analyzer,
    //                line_entry.line,
    //            );
    //            println!("Expected: {:?}", line_entry.analysis);
    //            println!("Got: {:?}", analysis);
    //            panic!("Analysis mismatch");
    //        }
    //    }
    //}
}
