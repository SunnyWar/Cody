// ...existing code...

impl Evaluator for NNUEEvaluator {
    fn evaluate(&self, pos: &Position) -> i32 {
        self.nnue.evaluate(pos)
    }
}
