use clap::Parser;
use miette::{IntoDiagnostic, Result};
use serde::Deserialize;
use std::{collections::HashMap, io::Read, path::PathBuf};

#[derive(Parser)]
struct Args {
    file: PathBuf,
}

#[derive(knuffel::Decode, Debug)]
struct Receipts {
    #[knuffel(children(name = "person"))]
    persons: Vec<Person>,
    #[knuffel(children(name = "receipt"))]
    receipts: Vec<Receipt>,
}

#[derive(knuffel::Decode, Debug)]
struct Receipt {
    #[knuffel(argument)]
    name: String,
    #[knuffel(property)]
    paid_by: String,
    #[knuffel(children)]
    items: Vec<Item>,
}

#[derive(knuffel::Decode, Debug)]
struct Item {
    #[knuffel(node_name)]
    name: String,
    #[knuffel(argument)]
    cost: f32,
    #[knuffel(arguments)]
    shared_by: Vec<String>,
}

#[derive(knuffel::Decode, Debug)]
struct Person {
    #[knuffel(argument)]
    name: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut file = std::fs::File::open(&args.file).into_diagnostic()?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).into_diagnostic()?;

    let buf = String::from_utf8_lossy(&buf);

    let db = knuffel::parse::<Receipts>(&args.file.to_string_lossy(), &buf)?;

    resolve(db);

    Ok(())
}

fn resolve(database: Receipts) {
    let mut balances: HashMap<String, HashMap<String, f32>> = database
        .persons
        .into_iter()
        .map(|person| (person.name, HashMap::new()))
        .collect();

    for receipt in database.receipts {
        for item in receipt.items.iter() {
            let shared_by: Vec<String> = {
                if item.shared_by.len() == 0 {
                    balances.keys().cloned().collect()
                } else {
                    item.shared_by.clone()
                }
            };
            let cost = item.cost / shared_by.len() as f32;

            for person in shared_by
                .into_iter()
                .filter(|person| person != &receipt.paid_by)
            {
                balances
                    .get_mut(&person)
                    .unwrap()
                    .entry(receipt.paid_by.clone())
                    .and_modify(|balance| *balance += cost)
                    .or_insert(cost);
            }
        }
    }

    for (person, debts) in balances {
        for (debt_to, balance) in debts {
            println!("{person} owes {balance} PLN to {debt_to}");
        }
    }
}
