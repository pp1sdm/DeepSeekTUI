use futures::StreamExt;
use super::Ui;

impl Ui {
    // 示波器状态更新
    pub fn update_scope_data(&mut self) {
        if self.scope_paused {
            return;
        }
        self.scope_tick += 1;

        let samples = self.graph_config.samples as usize;
        let tick = self.scope_tick;
        let phase = tick as f64 * 0.03;

        let ch0: Vec<f64> = (0..samples)
            .map(|i| {
                ((i as f64 * 0.03 + phase).sin() * 0.5
                    + (i as f64 * 0.07 + phase * 1.3).sin() * 0.3) * 0.6
            })
            .collect();

        let ch1: Vec<f64> = (0..samples)
            .map(|i| {
                ((i as f64 * 0.05 + phase * 1.7).sin() * 0.4
                    + (i as f64 * 0.11 + phase * 0.8).sin() * 0.2) * 0.6
            })
            .collect();

        self.scope_data = vec![ch0, ch1];
    }

    // 流式添加数据
    pub async fn poll_stream(&mut self) {
        let stream = match &mut self.stream {
            Some(s) => s,
            None => return,
        };

        match stream.next().await {
            Some(Ok(chunk)) => {
                self.session.append_to_last(&chunk);
            }
            Some(Err(e)) => {
                self.session.append_to_last(&format!("\n[错误: {}]", e));
                self.stream = None;
                self.session.finish();
            }
            None => {
                self.stream = None;
                self.session.finish();
            }
        }
    }
}