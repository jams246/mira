<?php

namespace App\Tests\Ampere\SystemInfo\Reader;

use App\Ampere\SystemInfo\Reader\CpuModelName;

class CpuModelNameTest extends ReaderHelper
{
    public function testParseResponseIsObjectWithModelNameAttribute(): void
    {
        $fixtureContent = $this->loadFixture('cpuinfo');

        $object = new CpuModelName();
        $response = $object->parse($fixtureContent);

        $this->assertIsObject($response);
        $this->assertObjectHasAttribute('model name', $response);
    }
}
