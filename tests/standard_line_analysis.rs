use bacon::*;

/// Check we reproduce all the line analyses from json files in standard_line_analysis directory
#[test]
fn test_standard_line_analysis() {
    let dir = file!().strip_suffix(".rs").unwrap();
    for entry in std::fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let (Some(name), Some(extension)) = (path.file_name(), path.extension()) else {
            continue;
        };
        if extension != "json" {
            continue;
        }
        let file = std::fs::File::open(&path).unwrap();
        let reader = std::io::BufReader::new(file);
        let export: AnalysisExport = serde_json::from_reader(reader).unwrap();
        for line_entry in export.lines {
            // checking that we reproduce the same analysis
            let analysis = LineAnalysis::from(&line_entry.line);
            if analysis != line_entry.analysis {
                println!("Wrong analysis in {:?} for {:#?}", name, line_entry.line);
                println!("Expected: {:?}", line_entry.analysis);
                println!("Got: {:?}", analysis);
                panic!("Analysis mismatch");
            }
        }
    }
}
