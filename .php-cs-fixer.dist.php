<?php

$finder = PhpCsFixer\Finder::create()
    ->in('src')
    ->in('tests');

$config = new PhpCsFixer\Config();
$config->setUsingCache(false)
    ->setRiskyAllowed(true)
    ->setRules([
        '@Symfony' => true,
        'native_function_invocation' => [
            'include' => ['@internal', '@compiler_optimized'], // Internal functions and Zend opcode optimized
            'scope' => 'namespaced', // Runs only on namespaced functions. The list can be got from get_defined_functions()
            'strict' => true // Remove backslash from functions that does not mean to have it
        ]
    ])
    ->setFinder($finder);

return $config;