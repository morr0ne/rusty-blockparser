use clap::{App, Arg, ArgMatches, SubCommand};
use serde::Serialize;
use std::path::PathBuf;
use std::{collections::HashMap, fs};

use crate::blockchain::proto::block::Block;
use crate::callbacks::Callback;
use crate::common::utils;
use crate::errors::OpResult;
use crate::{blockchain::parser::types::CoinType, common::utils::arr_to_hex_swapped};

use super::common;

/// Dumps the whole blockchain into json files
pub struct JsonDump {
    dump_folder: PathBuf,
    start_height: u64,
    end_height: u64,
    tx_count: u64,
    in_count: u64,
    out_count: u64,
    parsed_blocks: Vec<ParsedBlock>,
}

#[derive(Serialize)]
pub struct ParsedBlock {
    pub block_hash: String,
    pub timestamp: u32,
    pub tx_in: Vec<Transaction>,
    pub tx_out: Vec<Transaction>,
}

#[derive(Serialize)]
pub struct Transaction {
    pub tx: String,
    pub address: Option<String>,
}

impl Callback for JsonDump {
    fn build_subcommand<'a, 'b>() -> App<'a, 'b>
    where
        Self: Sized,
    {
        SubCommand::with_name("jsondump")
            .about("Dumps the whole blockchain into Json files")
            .version("0.1")
            .author("gcarq <egger.m@protonmail.com>")
            .arg(
                Arg::with_name("dump-folder")
                    .help("Folder to store csv files")
                    .index(1)
                    .required(true),
            )
    }

    fn new(matches: &ArgMatches) -> OpResult<Self>
    where
        Self: Sized,
    {
        let dump_folder = &PathBuf::from(matches.value_of("dump-folder").unwrap());
        let cb = JsonDump {
            dump_folder: PathBuf::from(dump_folder),
            start_height: 0,
            end_height: 0,
            tx_count: 0,
            in_count: 0,
            out_count: 0,
            parsed_blocks: vec![],
        };
        Ok(cb)
    }

    fn on_start(&mut self, _: &CoinType, block_height: u64) -> OpResult<()> {
        self.start_height = block_height;
        info!(target: "callback", "Using `jsondump` with dump folder: {} ...", &self.dump_folder.display());
        Ok(())
    }

    fn on_block(&mut self, block: &Block, block_height: u64) -> OpResult<()> {
        let block_hash = utils::arr_to_hex_swapped(&block.header.hash);
        let timestamp = block.header.value.timestamp;
        let mut tx_in: Vec<Transaction> = vec![];
        let mut tx_out: Vec<Transaction> = vec![];

        for tx in &block.txs {
            let txid = arr_to_hex_swapped(&tx.hash);
            for input in &tx.value.inputs {
                tx_in.push(Transaction {
                    tx: txid.to_owned(),
                    address: None,
                })
            }

            for output in &tx.value.outputs {
                // output.script.address;
                tx_out.push(Transaction {
                    tx: txid.to_owned(),
                    address: output.script.address.to_owned(),
                })
            }
        }

        self.parsed_blocks.push(ParsedBlock {
            block_hash,
            timestamp,
            tx_in,
            tx_out,
        });

        Ok(())
    }

    fn on_complete(&mut self, block_height: u64) -> OpResult<()> {
        self.end_height = block_height;

        fs::write(
            self.dump_folder.join("blocks.json"),
            &serde_json::to_vec(&self.parsed_blocks).expect("json failed"),
        )?;

        info!(target: "callback", "Done.\nDumped all {} blocks:\n\
                                   \t-> transactions: {:9}\n\
                                   \t-> inputs:       {:9}\n\
                                   \t-> outputs:      {:9}",
             self.end_height, self.tx_count, self.in_count, self.out_count);
        Ok(())
    }
}
