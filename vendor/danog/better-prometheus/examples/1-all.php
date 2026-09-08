<?php

require 'vendor/autoload.php';

use danog\BetterPrometheus\BetterCollectorRegistry;
use Prometheus\Storage\InMemory;
use Prometheus\Storage\Redis;

$adapter = new InMemory;
// Any other promphp adapter may also be used...
// $adapter = new Redis();

$registry = new BetterCollectorRegistry($adapter);

// Note the difference with promphp: the labels are keys => values, not just keys.  
$counter = $registry->getOrRegisterCounter(
    'test',
    'some_counter',
    'it increases',
    // Note: these are default label key+values, they will be sent verbatim, no changes
    ['someLabel' => 'defaultValue']
);

// Specify some additional labels post-construction like this (both keys and values)...
$counter->incBy(3, ['type' => 'blue']);

// ...or add some more default labels to the counter, creating a new counter:
$newCounter = $counter->addLabels(['someOtherLabel' => 'someOtherDefaultValue']);
assert($newCounter !== $counter); // true
$newCounter->incBy(3, ['type' => 'blue']);



// Gauges can also be used
$gauge = $registry->getOrRegisterGauge(
    'test',
    'some_gauge',
    'it sets',
    ['someLabel' => 'defaultValue']
);
$gauge->set(2.5, ['type' => 'blue']);



// As well as histograms
$histogram = $registry->getOrRegisterHistogram(
    'test',
    'some_histogram',
    'it observes',
    ['someLabel' => 'defaultValue'],
    // [0.1, 1, 2, 3.5, 4, 5, 6, 7, 8, 9]
);
$histogram->observe(3.5, ['type' => 'blue']);


// And suummaries
$summary = $registry->getOrRegisterSummary(
    'test',
    'some_summary',
    'it observes a sliding window',
    ['someLabel' => 'defaultValue'],
    // 84600,
    // [0.01, 0.05, 0.5, 0.95, 0.99]
);

$summary->observe(5, ['type' => 'blue']);
