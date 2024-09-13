use crate::framework::application::Context;
use crate::Result;

pub trait WorkerTrait {
    fn intialize(&mut self, ctx: Context) -> Result<()>;
    fn do_work(&mut self, ctx: Context) -> Result<()>;
    fn deinitialize(&mut self, ctx: Context) -> Result<()>;
    fn process_events(&mut self) -> Result<()>;
}