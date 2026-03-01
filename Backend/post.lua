wrk.method = "POST"
wrk.headers["Content-Type"] = "application/json"

local counter = 0

request = function()
    counter = counter + 1
    local body = '{"message":"hello ' .. counter .. '","token":"MLrxfuDiNQlLOgJxGVaIBtOhetkl2lbXhQgKDepIYAM"}'
    return wrk.format("POST", nil, nil, body)
end