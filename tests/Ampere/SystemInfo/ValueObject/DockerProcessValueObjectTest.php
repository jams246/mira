<?php

namespace App\Tests\Ampere\SystemInfo\ValueObject;

use App\Ampere\SystemInfo\ValueObject\DockerProcessValueObject;
use App\Ampere\SystemInfo\ValueObject\Exception\ValueObjectException;
use PHPUnit\Framework\TestCase;

class DockerProcessValueObjectTest extends TestCase
{
    public function testInvalidValueThrowsValueObjectException(): void
    {
        $this->expectException(ValueObjectException::class);

        (new DockerProcessValueObject('Test1', 'Test2', -1.0, 246, 1638759482));
    }

    /**
     * @dataProvider validDockerProcessValueProvider
     */
    public function testGetValues(string $name, string $state, float $cpu, int $memory, float $upTime, array $expected): void
    {
        $object = new DockerProcessValueObject($name, $state, $cpu, $memory, $upTime);

        $this->assertSame($expected[0], $object->getName());
        $this->assertSame($expected[1], $object->getState());
        $this->assertSame($expected[2], $object->getCpu());
        $this->assertSame($expected[3], $object->getMemory());
        $this->assertSame($expected[4], $object->getUpTime());
    }

    public function validDockerProcessValueProvider(): array
    {
        return [
            [
                'name',
                'state',
                1.0,
                246,
                1638759482.0,
                ['name', 'state', 1.0, 246, 1638759482.0],
            ],
        ];
    }
}
