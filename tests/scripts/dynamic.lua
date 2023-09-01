function printf(...)
  print(string.format(...))
end

printf('Hello from Lua!')

-- print out something passed by SWS (Rust)
printf('Your OS is "%s"', RUST_OS)
