<?php

namespace App\Tests\Ampere\SystemInfo\Reader;

use App\Ampere\SystemInfo\Reader\Processes;
use Symfony\Component\Cache\Adapter\AdapterInterface;

class ProcessTest extends ReaderHelper
{
    private AdapterInterface $cache;

    public function setUp(): void
    {
        $this->cache = $this->createMock(AdapterInterface::class);

        parent::setUp();
    }

    public function testParseReturnsObjectWithAttributes(): void
    {
        $fixtureContent = $this->loadFixture('process_stat');

        $object = new Processes($this->cache);

        $response = (object) $object->parse($fixtureContent);

        $this->assertIsObject($response);
        $this->assertObjectHasAttribute('State', $response);
        $this->assertObjectHasAttribute('Name', $response);
        $this->assertObjectHasAttribute('Pid', $response);
    }
}
