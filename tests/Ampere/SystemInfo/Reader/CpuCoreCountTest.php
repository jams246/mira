<?php

namespace App\Tests\Ampere\SystemInfo\Reader;

use App\Ampere\SystemInfo\Reader\CpuCoreCount;

class CpuCoreCountTest extends ReaderHelper
{
    public function testParseResponseIsArrayWithProcessorAttribute(): void
    {
        $fixtureContent = $this->loadFixture('cpuinfo');

        $object = new CpuCoreCount();
        $response = $object->parse($fixtureContent);

        $this->assertIsArray($response);
        $this->assertContains('processor', $response[1]);
    }
}
