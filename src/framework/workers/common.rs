
pub trait WorkerTrait {
    fn intialize(&mut self, ctx: ApplicationContext) -> Result<()>;
    fn do_work(&mut self, ctx: ApplicationContext) -> Result<()>;
    fn deinitialize(&mut self, ctx: ApplicationContext) -> Result<()>;
    fn process_events(&mut self) -> Result<()>;
}