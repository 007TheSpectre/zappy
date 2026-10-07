use crate::command::Command;
use ordered_list::OrderedList;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Per-client command buffer cap. The protocol allows at most this many queued
/// commands per player; anything beyond it is dropped (see `is_client_queue_full`).
pub const MAX_PENDING_PER_CLIENT: u32 = 10;

#[derive(Debug, Clone)]
pub struct ScheduledCommand {
    pub command: Command,
    pub execute_at: Instant,
    pub client_id: u32,
}

pub struct TimeManager {
    queue: OrderedList<ScheduledCommand, Instant, fn(&ScheduledCommand) -> Instant>,
    frequency: u32,
    /// Number of not-yet-executed commands queued per client, kept in sync with
    /// `queue` so the per-client cap check is O(1) instead of scanning the queue.
    pending_per_client: HashMap<u32, u32>,
}

impl TimeManager {
    /// Create a new time manager
    /// frequency scales command duration (default 100, higher = faster)
    pub fn new(frequency: u32) -> Self {
        Self {
            queue: OrderedList::new_by(|cmd| cmd.execute_at),
            frequency,
            pending_per_client: HashMap::new(),
        }
    }

    /// True when the client already has the maximum number of queued commands.
    /// Callers should drop further commands from this client without parsing them.
    pub fn is_client_queue_full(&self, client_id: u32) -> bool {
        self.pending_per_client
            .get(&client_id)
            .copied()
            .unwrap_or(0)
            >= MAX_PENDING_PER_CLIENT
    }

    /// Update the frequency used to scale command durations. Only commands
    /// scheduled after this call use the new value; already queued commands keep
    /// their absolute execution time.
    pub fn set_frequency(&mut self, frequency: u32) {
        self.frequency = frequency;
    }

    /// Schedule a command for execution
    /// Returns when the command will execute
    pub fn schedule(&mut self, command: Command, client_id: u32) -> Instant {
        let duration_units = command.execution_time() as u64;
        // Convert duration to milliseconds: ms = (action * 1000) / frequency
        // Example: action=7, frequency=100 -> ms = 7*1000/100 = 70 ms
        let ms = (duration_units * 1000) / self.frequency as u64;
        let execute_at = Instant::now() + Duration::from_millis(ms);

        self.queue.push(ScheduledCommand {
            command,
            execute_at,
            client_id,
        });
        *self.pending_per_client.entry(client_id).or_insert(0) += 1;
        execute_at
    }

    /// Check current time and return all commands ready to execute
    pub fn tick(&mut self) -> Vec<ScheduledCommand> {
        let now = Instant::now();
        let mut ready = Vec::new();

        while let Some(next) = self.queue.get(0) {
            if next.execute_at <= now {
                let scheduled = self.queue.remove(0);
                if let Some(count) = self.pending_per_client.get_mut(&scheduled.client_id) {
                    *count = count.saturating_sub(1);
                }
                ready.push(scheduled);
            } else {
                break;
            }
        }

        ready
    }

    pub fn next_timeout_ms(&self) -> Option<i32> {
        let next = self.queue.get(0)?;
        let now = Instant::now();
        let delay = next.execute_at.saturating_duration_since(now);
        let ms = delay.as_millis().min(i32::MAX as u128);
        Some(ms as i32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn schedule_then_execute() {
        let mut manager = TimeManager::new(100);
        manager.schedule(Command::Inventory, 0);
        manager.schedule(Command::Forward, 1);

        // First tick: nothing ready yet
        let ready = manager.tick();
        assert!(ready.is_empty());

        // Wait for Inventory to be ready (~10ms)
        thread::sleep(Duration::from_millis(20));
        let ready = manager.tick();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].client_id, 0);

        // Wait for Forward to be ready (~70ms total)
        thread::sleep(Duration::from_millis(100));
        let ready = manager.tick();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].client_id, 1);
    }

    #[test]
    fn multiple_commands_same_execution_time() {
        let mut manager = TimeManager::new(100);
        manager.schedule(Command::Inventory, 0);
        manager.schedule(Command::Inventory, 1);
        manager.schedule(Command::Inventory, 2);

        thread::sleep(Duration::from_millis(20));
        let ready = manager.tick();
        assert_eq!(ready.len(), 3);
    }

    #[test]
    fn set_frequency_changes_future_scheduling() {
        let mut manager = TimeManager::new(100);
        let slow = manager.schedule(Command::Forward, 0); // 70ms at freq 100
        manager.set_frequency(400);
        let fast = manager.schedule(Command::Forward, 1); // ~17ms at freq 400
        // Raising the frequency makes the same command schedule sooner; the
        // ~53ms gap dwarfs the few microseconds between the two now() calls.
        assert!(fast < slow);
    }

    #[test]
    fn frequency_affects_timing() {
        let mut slow = TimeManager::new(50);
        let mut fast = TimeManager::new(200);

        let slow_time = slow.schedule(Command::Forward, 0);
        let fast_time = fast.schedule(Command::Forward, 0);

        // Slow should take longer
        assert!(slow_time > fast_time);
    }

    #[test]
    fn commands_ordered_by_execution_time() {
        let mut manager = TimeManager::new(100);
        manager.schedule(Command::Forward, 0);
        manager.schedule(Command::Inventory, 1);
        manager.schedule(Command::Incantation, 2);

        thread::sleep(Duration::from_millis(20));
        let ready = manager.tick();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].client_id, 1); // Inventory executes first

        thread::sleep(Duration::from_millis(100));
        let ready = manager.tick();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].client_id, 0); // Forward executes second
    }
}
