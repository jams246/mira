<?php

namespace App\Tests\Ampere\SystemInfo\Reader;

use App\Ampere\SystemInfo\Reader\BitVersion;

class BitVersionTest extends ReaderHelper
{
    public function testParseResponseIsObjectWithFlagAttribute(): void
    {
        $fixtureContent = $this->loadFixture('cpuinfo');

        $object = new BitVersion();
        $response = $object->parse($fixtureContent);

        $this->assertIsObject($response);
        $this->assertObjectHasAttribute('flags', $response);
    }
}
