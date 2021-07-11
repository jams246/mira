<?php

namespace App\Tests\Ampere\SystemInfo\Reader;

use PHPUnit\Framework\TestCase;

abstract class ReaderHelper extends TestCase
{
    private const FIXTURE_PATH = __DIR__.'/Fixtures/';

    /**
     * @throws \Exception
     */
    protected function loadFixture(string $fixtureName): string
    {
        if (!\file_exists(self::FIXTURE_PATH.$fixtureName)) {
            throw new \Exception(\sprintf('Fixture `%s` not found in Fixtures directory.', $fixtureName));
        }

        $fileContents = \file_get_contents(self::FIXTURE_PATH.$fixtureName);
        if (false === $fileContents || 0 === \strlen($fileContents)) {
            throw new \Exception(\sprintf('Fixture `%s` is empty.', $fixtureName));
        }

        return $fileContents;
    }
}
