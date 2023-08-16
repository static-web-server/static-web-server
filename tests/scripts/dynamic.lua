-- just a busy-waiting function
function sleep(secs)
  local cpu_secs = os.clock()
  while os.clock() - cpu_secs <= secs do end
end

function printf(...)
  print(string.format(...))
end

printf('Hello from Lua!')

sleep(2)

-- print out something passed by SWS (Rust)
printf('Your OS is "%s"', RUST_OS)
