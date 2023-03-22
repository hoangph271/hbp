module.exports = {
  apps : [{
    name: "hbp",
    script: "cargo",
    args: "run --release",
    watch: true,
    ignore_watch: ["target"],
    watch_options: {
      followSymlinks: false
    },
  }]
}
