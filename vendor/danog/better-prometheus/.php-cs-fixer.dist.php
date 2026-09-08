<?php

$config = new class extends Amp\CodeStyle\Config {
    public function getRules(): array
    {
        return array_merge(parent::getRules(), [
            'void_return' => true,
            'phpdoc_to_param_type' => true,
            'phpdoc_to_return_type' => true,
            'phpdoc_to_property_type' => true,
        ]);
    }
};

$config->getFinder()
    ->in(__DIR__ . '/lib');

$cacheDir = getenv('TRAVIS') ? getenv('HOME') . '/.php-cs-fixer' : __DIR__;

$config->setCacheFile($cacheDir . '/.php_cs.cache');

return $config;
