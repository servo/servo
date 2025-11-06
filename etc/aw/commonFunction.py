import re
from decimal import Decimal

def calcuteFrameRate(file_name):
    """
    calcuteFrameRate
    :param file_name:
    :return:
    """
    commands_list = []
    # 打开文件.trace
    with open(file_name, 'r') as f:
        for line in f.readlines():
            if len(re.findall(r'org.servo.servo', line)) > 0 and len(re.findall(r'H:SendCommands', line)) > 0:
                commands_list.append(line)

    # calcute time
    start_time = commands_list[0].split(' ')[9].split(':')[0]
    end_time = commands_list[-1].split(' ')[9].split(':')[0]

    interval_time = Decimal(end_time) - Decimal(start_time)
    if round(float(len(commands_list) / interval_time), 2) > 120.00:
        return 120.00
    else:
        return round(float(len(commands_list) / interval_time), 2)


if __name__ == '__main__':
    frame_rate = calcuteFrameRate(r"../traces/my_trace.html")


