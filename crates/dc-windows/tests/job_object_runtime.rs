//! Runtime Job Object proof: creates a real Win32 Job Object, spawns a
//! process, assigns it to the job, sets resource limits, and terminates it.
//!
//! This is NOT a synthetic test — it exercises real Win32 APIs on Windows.
//! It is `#[ignore]` by default and must be run explicitly:
//!
//! ```text
//! cargo test -p dc-windows --test job_object_runtime -- --ignored
//! ```

#[cfg(windows)]
mod job_runtime_tests {
    use dc_windows::job::JobLimits;
    use dc_windows::job_os::OsJobObject;

    /// Create a real Job Object and set kill-on-close + process limit.
    #[test]
    #[ignore]
    fn create_job_object_and_set_limits() {
        let job = OsJobObject::create().expect("must create job object");
        let limits = JobLimits {
            max_processes: 4,
            max_memory_bytes: 64 * 1024 * 1024,      // 64 MB
            max_working_set_bytes: 32 * 1024 * 1024, // 32 MB
            max_cpu_time_100ns: 0,                   // no CPU time limit
        };
        job.set_limits(&limits).expect("must set limits");
        println!("Job Object created and limits set successfully");
    }

    /// Spawn a child process, adopt it into a Job Object, and terminate it.
    #[test]
    #[ignore]
    fn adopt_process_into_job_and_terminate() {
        // Spawn a simple child process (cmd /c exit 0)
        let child = std::process::Command::new("cmd.exe")
            .args(["/c", "exit", "0"])
            .spawn()
            .expect("must spawn child");
        let pid = child.id();
        println!("Spawned child PID: {}", pid);

        let job = OsJobObject::create().expect("must create job object");
        let limits = JobLimits {
            max_processes: 2,
            max_memory_bytes: 0, // no memory limit
            max_working_set_bytes: 0,
            max_cpu_time_100ns: 0,
        };
        job.set_limits(&limits).expect("must set limits");

        // Assign the child to the job
        job.assign_process(pid).expect("must assign process to job");
        println!("Child PID {} assigned to job", pid);

        // Terminate the job (kills the child)
        job.terminate_all(1).expect("must terminate job");
        println!("Job terminated (kill-on-close will clean up any remaining)");

        // Wait for child to exit (it should already be dead)
        let status = child.wait_with_output().expect("must wait for child");
        println!("Child exited with status: {:?}", status.status);
    }

    /// Verify that a process assigned to a job is killed when the job terminates.
    #[test]
    #[ignore]
    fn terminate_via_job_kills_process() {
        // Spawn a long-running child
        let child = std::process::Command::new("cmd.exe")
            .args(["/c", "timeout", "/t", "60", "/nobreak"])
            .spawn()
            .expect("must spawn child");
        let pid = child.id();
        println!("Spawned long-running child PID: {}", pid);

        let job = OsJobObject::create().expect("must create job object");
        let limits = JobLimits {
            max_processes: 1,
            max_memory_bytes: 0,
            max_working_set_bytes: 0,
            max_cpu_time_100ns: 0,
        };
        job.set_limits(&limits).expect("must set limits");
        job.assign_process(pid).expect("must assign");

        // Terminate the job — this must kill the child
        job.terminate_all(42).expect("must terminate job");

        // The child should exit with a non-zero code (killed by job)
        let output = child.wait_with_output().expect("must wait for child");
        println!("Child after job termination: status={:?}", output.status);
        assert!(
            !output.status.success(),
            "child must not exit successfully after job termination"
        );
    }
}
