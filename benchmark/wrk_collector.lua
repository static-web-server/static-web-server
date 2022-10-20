-- Script that runs at wrk's "done" stage collecting the stats in JSON and CSV formats.
-- https://github.com/wg/wrk/blob/master/SCRIPTING
done = function(summary, latency, requests)
    local json = string.format("{\n")
    json = json .. string.format('\t"requests": %d,\n', summary.requests)
    json = json .. string.format('\t"duration_ms": %0.2f,\n', summary.duration / 1000)
    json = json .. string.format('\t"requests_per_sec": %0.2f,\n', (summary.requests / summary.duration) * 1e6)
    json = json .. string.format('\t"bytes": %d,\n', summary.bytes)
    json = json .. string.format('\t"bytes_transfer_per_sec": %0.2f,\n', (summary.bytes / summary.duration) * 1e6)
    json = json .. string.format('\t"connect_errors": %d,\n', summary.errors.connect)
    json = json .. string.format('\t"read_errors": %d,\n', summary.errors.read)
    json = json .. string.format('\t"write_errors": %d,\n', summary.errors.write)
    json = json .. string.format('\t"http_errors": %d,\n', summary.errors.status)
    json = json .. string.format('\t"timeouts": %d,\n', summary.errors.timeout)
    json = json .. string.format('\t"latency_min": %0.2f,\n', latency.min)
    json = json .. string.format('\t"latency_max": %0.2f,\n', latency.max)
    json = json .. string.format('\t"latency_mean_ms": %0.2f,\n', latency.mean / 1000)
    json = json .. string.format('\t"latency_stdev": %0.2f,\n', latency.stdev)

    json = json .. string.format('\t"latency_distribution": [\n')
    for _, pair in pairs({50, 75, 90, 99}) do
        json = json .. string.format("\t\t{\n")

        local percent = latency:percentile(pair)
        json = json .. string.format('\t\t\t"percentile": %g,\n\t\t\t"latency_ms": %0.2f\n', pair, percent / 1000)
        json = json .. string.format("\t\t}%s\n", pair > 90 and "" or ",")
    end

    json = json .. string.format("\t]\n}\n")

    local file, err = io.open("benchmark/wrk_results.json", "w")
    if file then
        file:write(json)
        file:close()
    else
        print("error saving json results file:", err)
    end

    local csv = ''
    csv = csv .. string.format('requests,')
    csv = csv .. string.format('duration_ms,')
    csv = csv .. string.format('requests_per_sec,')
    csv = csv .. string.format('bytes,')
    csv = csv .. string.format('bytes_transfer_per_sec,')
    csv = csv .. string.format('connect_errors,')
    csv = csv .. string.format('read_errors,')
    csv = csv .. string.format('write_errors,')
    csv = csv .. string.format('http_errors,')
    csv = csv .. string.format('timeouts,')
    csv = csv .. string.format('latency_min,')
    csv = csv .. string.format('latency_max,')
    csv = csv .. string.format('latency_mean_ms,')
    csv = csv .. string.format('latency_stdev\n')

    csv = csv .. string.format('%d,', summary.requests)
    csv = csv .. string.format('%0.2f,', summary.duration / 1000)
    csv = csv .. string.format('%0.2f,', (summary.requests / summary.duration) * 1e6)
    csv = csv .. string.format('%d,', summary.bytes)
    csv = csv .. string.format('%0.2f,', (summary.bytes / summary.duration) * 1e6)
    csv = csv .. string.format('%d,', summary.errors.connect)
    csv = csv .. string.format('%d,', summary.errors.read)
    csv = csv .. string.format('%d,', summary.errors.write)
    csv = csv .. string.format('%d,', summary.errors.status)
    csv = csv .. string.format('%d,', summary.errors.timeout)
    csv = csv .. string.format('%0.2f,', latency.min)
    csv = csv .. string.format('%0.2f,', latency.max)
    csv = csv .. string.format('%0.2f,', latency.mean / 1000)
    csv = csv .. string.format('%0.2f\n', latency.stdev)

    local file, err = io.open("benchmark/wrk_results.csv", "w")
    if file then
        file:write(csv)
        file:close()
    else
        print("error saving csv results file:", err)
    end
end
