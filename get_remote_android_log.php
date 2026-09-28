<?php

$ch = curl_init('http://192.168.2.118:8088/api/logs?file=debug.log');
curl_setopt_array($ch, [
    CURLOPT_RETURNTRANSFER => true,
    CURLOPT_TIMEOUT => 10,
]);
$res = curl_exec($ch);
curl_close($ch);

$lines = explode("\n", (string)$res);
$tail = array_slice($lines, -50);
echo "=== LOGS FROM 192.168.2.118:8088 (LAST 50 LINES) ===\n";
echo implode("\n", $tail);
