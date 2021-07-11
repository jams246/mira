<?php

namespace App\Tests\Ampere\SystemInfo\Reader;

use App\Ampere\SystemInfo\Reader\Memory;

class MemoryTest extends ReaderHelper
{
    public function testParseReturnsObjectWithAttributes(): void
    {
        $fixtureContent = $this->loadFixture('meminfo');

        $object = new Memory();
        $response = (object) $object->parse($fixtureContent);

        $this->assertIsObject($response);
        $this->assertObjectHasAttribute('MemTotal', $response);
        $this->assertObjectHasAttribute('MemFree', $response);
        $this->assertObjectHasAttribute('MemAvailable', $response);
        $this->assertObjectHasAttribute('Buffers', $response);
        $this->assertObjectHasAttribute('Cached', $response);
    }

    public function testCalculateMemoryUsage(): void
    {
        $fixtureContent = $this->loadFixture('meminfo');

        $object = new Memory();
        $response = (object) $object->parse($fixtureContent);

        $response = $object->calculateMemoryUsage($response);

        $this->assertIsFloat($response);
        $this->assertSame(38.0, $response);
    }
}
