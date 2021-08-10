#! /usr/bin/python3
# Licensing: See the LICENSE file

import sys, os
from math import log
from typing import List

def printHelp():
	print("Usage: ./opensesTAme.py func filename")
	print("Where func is one of:")
	print("\thelp - prints this message")
	print("\tdump_bootlogs - dumps boot logs (TA stores up to ten of these)")
	print("\tdump_sqlitedb - dumps the internal SQLite database")
	print("\tshow_buildid - shows build number")
	print("\tshow_serial - shows serial number")

numArgs: int = len(sys.argv)
if(numArgs < 3):
	printHelp()
	exit()

taFilename: str = str(sys.argv[2])
TA_EXPECTED_SIZE_BYTES = 2097152

bootlog_offsets = [
	0, # Zero-element
	0x2A22E, #1
	0x2DA22, #2
	0x31CEE, #3
	0x3542A, #4
	0x38C46, #5
	0x3C7A2, #6
	0x65412, #7
	0x68C2E, #8
	0x6C78A, #9
	0x70A2E, #10
]
version_offset = 0x7B4
serial_offset = 0x600B4
sqlitedb_offset = 0x20044

if(sys.argv[1] == 'help'):
	printHelp()
	exit()

print("Opening file: " + taFilename)

# We never want to write to TA (unless you have a magic device)
taFile = open(taFilename, 'rb')
taFileSize: int = os.path.getsize(taFilename)

print("TA size: " + str(taFileSize) + " bytes")

if not (taFileSize == TA_EXPECTED_SIZE_BYTES):
	print("TA size mismatch! Is your dump corrupted?")
	exit()

print ("TA size in tact, proceeding..\n")

def dump_bootlogs():
	bootlog: List[str] = [""]*11 # We have 10 bootlogs but want to keep the indices sane
	for i in range(1, 11):
		print("Dumping bootlog "+ str(i) + " at " + str(bootlog_offsets[i]))

		# Reset the pointer back to the beginning of the file
		bootlog[i] = taFile.seek(0)
		bootlog[i] = taFile.seek(bootlog_offsets[1])
		bootlog[i] = taFile.read(14309)
		bootlog[i] = bootlog[i].decode('utf-8')

		tempFilename: str = "bootlogs/bootlog" + str(i) + ".txt"
		print("Saving to " + tempFilename + "..")
		tempFile = open(tempFilename, 'w+')
		tempFile.write(bootlog[i])
		tempFile.close()

def show_build():
	# Reset the pointer back to the beginning of the file
	ret = taFile.seek(0)
	ret = taFile.seek(version_offset)
	ret = taFile.read(32).decode('utf-8') # 32 is an educated guess, it was 29 on Tama-Akari
	print("Image version: " + str(ret))

def show_serialno():
	# Reset the pointer back to the beginning of the file
	ret = taFile.seek(0)
	ret = taFile.seek(serial_offset)
	ret = taFile.read(10).decode('utf-8')
	print("Serial no.: " + str(ret))

def dump_sqlitedb():
	# Reset the pointer back to the beginning of the file
	ret = taFile.seek(0)
	ret = taFile.seek(sqlitedb_offset+16) #(as per sqlite spec https://www.sqlite.org/fileformat.html)
	ret = taFile.read(2)
	ret = int.from_bytes(ret, "little") # aarch64le
	print("SQLite DB size: 2^" + str(ret) + " (" + str(2**ret) + " B)")
	sqlitedb_len: int = int(2**ret)

	# Reset the pointer back to the beginning of the file
	ret = taFile.seek(0)
	ret = taFile.seek(sqlitedb_offset)
	ret = taFile.read(sqlitedb_len) #100 bytes as per sqlite spec
	tempFile = open("sqlite.db", "wb+")
	tempFile.write(ret)
	sqlitedb_filesize = os.path.getsize("sqlite.db")
	if not (sqlitedb_len == sqlitedb_filesize):
		print("SQLite DB size mismatched! Expected: " + str(sqlitedb_len) + ", got: " + str(sqlitedb_filesize))
	tempFile.close()
	print("Saved results to sqlite.db!")


if(sys.argv[1] == "dump_bootlogs"):
	dump_bootlogs()
	exit()
elif(sys.argv[1] == "dump_sqlitedb"):
	dump_sqlitedb()
	exit()
elif(sys.argv[1] == "show_buildid"):
	show_build()
	exit()
elif(sys.argv[1] == "show_serial"):
	show_serialno()
	exit()

taFile.close()
